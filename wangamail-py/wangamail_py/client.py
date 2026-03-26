"""Graph mail client implementation for wangamail-py."""

from __future__ import annotations

import os
import time
from dataclasses import dataclass
from urllib.parse import quote

import json
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

from .errors import ConfigError, GraphAPIError
from .models import SendMailRequest

DEFAULT_SCOPE = "https://graph.microsoft.com/.default"
DEFAULT_GRAPH_BASE = "https://graph.microsoft.com/v1.0"


@dataclass(slots=True)
class _TokenState:
    access_token: str | None = None
    expires_at: float = 0.0


class GraphMailClient:
    """Client for sending email via Microsoft Graph using app credentials."""

    def __init__(
        self,
        *,
        tenant_id: str,
        client_id: str,
        client_secret: str,
        token_url: str | None = None,
        graph_base: str = DEFAULT_GRAPH_BASE,
        scope: str = DEFAULT_SCOPE,
        timeout: float = 20.0,
    ) -> None:
        if not tenant_id:
            raise ConfigError("tenant_id is required")
        if not client_id:
            raise ConfigError("client_id is required")
        if not client_secret:
            raise ConfigError("client_secret is required")

        self._tenant_id = tenant_id
        self._client_id = client_id
        self._client_secret = client_secret
        self._token_url = token_url or f"https://login.microsoftonline.com/{tenant_id}/oauth2/v2.0/token"
        self._graph_base = graph_base.rstrip("/")
        self._scope = scope
        self._timeout = timeout
        self._token_state = _TokenState()

    @classmethod
    def from_env(cls) -> "GraphMailClient":
        """Create a client from AZURE_* environment variables."""
        return cls(
            tenant_id=os.environ.get("AZURE_TENANT_ID", ""),
            client_id=os.environ.get("AZURE_CLIENT_ID", ""),
            client_secret=os.environ.get("AZURE_CLIENT_SECRET", ""),
        )

    def _get_token(self) -> str:
        now = time.time()
        if self._token_state.access_token and now < self._token_state.expires_at:
            return self._token_state.access_token

        form_data = (
            "client_id="
            + quote(self._client_id, safe="")
            + "&client_secret="
            + quote(self._client_secret, safe="")
            + "&scope="
            + quote(self._scope, safe="")
            + "&grant_type=client_credentials"
        )
        req = Request(
            self._token_url,
            data=form_data.encode("utf-8"),
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            method="POST",
        )
        try:
            with urlopen(req, timeout=self._timeout) as response:
                payload = json.loads(response.read().decode("utf-8"))
        except HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace")
            raise GraphAPIError(f"token request failed: {exc.code} {body}") from exc
        except URLError as exc:
            raise GraphAPIError(f"token request failed: {exc}") from exc

        token = payload.get("access_token")
        expires_in = int(payload.get("expires_in", 300))
        if not token:
            raise GraphAPIError("token response did not include access_token")

        self._token_state = _TokenState(
            access_token=token,
            expires_at=now + max(expires_in - 60, 30),
        )
        return token

    def send_mail(self, from_user: str, request: SendMailRequest) -> None:
        """Send an email as the given user (user id or userPrincipalName)."""
        if not from_user:
            raise ConfigError("from_user is required")

        token = self._get_token()
        url = f"{self._graph_base}/users/{quote(from_user, safe='')}/sendMail"

        payload = json.dumps(request.to_graph()).encode("utf-8")
        req = Request(
            url,
            data=payload,
            headers={
                "Authorization": f"Bearer {token}",
                "Content-Type": "application/json",
            },
            method="POST",
        )
        try:
            with urlopen(req, timeout=self._timeout) as response:
                status = getattr(response, "status", response.getcode())
        except HTTPError as exc:
            body = exc.read().decode("utf-8", errors="replace")
            raise GraphAPIError(f"sendMail failed: {exc.code} {body}") from exc
        except URLError as exc:
            raise GraphAPIError(f"sendMail failed: {exc}") from exc

        if status != 202:
            raise GraphAPIError(f"sendMail failed: {status}")
