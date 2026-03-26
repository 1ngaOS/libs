"""wangamail-py package exports."""

from .client import GraphMailClient
from .errors import ConfigError, GraphAPIError, WangaMailError
from .models import Message, MessageBody, Recipient, SendMailRequest

__all__ = [
    "GraphMailClient",
    "WangaMailError",
    "ConfigError",
    "GraphAPIError",
    "Message",
    "MessageBody",
    "Recipient",
    "SendMailRequest",
]
