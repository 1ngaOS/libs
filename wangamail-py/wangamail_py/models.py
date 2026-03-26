"""Data models for Microsoft Graph sendMail payloads."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Dict, List, Literal

BodyType = Literal["Text", "HTML"]


@dataclass(slots=True)
class Recipient:
    email: str

    @classmethod
    def from_email(cls, email: str) -> "Recipient":
        return cls(email=email)

    def to_graph(self) -> Dict[str, Dict[str, str]]:
        return {"emailAddress": {"address": self.email}}


@dataclass(slots=True)
class MessageBody:
    content_type: BodyType
    content: str

    def to_graph(self) -> Dict[str, str]:
        return {"contentType": self.content_type, "content": self.content}


@dataclass(slots=True)
class Message:
    subject: str
    body: MessageBody
    to_recipients: List[Recipient] = field(default_factory=list)
    cc_recipients: List[Recipient] = field(default_factory=list)
    bcc_recipients: List[Recipient] = field(default_factory=list)

    def to_graph(self) -> Dict[str, object]:
        payload: Dict[str, object] = {
            "subject": self.subject,
            "body": self.body.to_graph(),
            "toRecipients": [r.to_graph() for r in self.to_recipients],
        }
        if self.cc_recipients:
            payload["ccRecipients"] = [r.to_graph() for r in self.cc_recipients]
        if self.bcc_recipients:
            payload["bccRecipients"] = [r.to_graph() for r in self.bcc_recipients]
        return payload


@dataclass(slots=True)
class SendMailRequest:
    message: Message
    save_to_sent_items: bool = True

    def to_graph(self) -> Dict[str, object]:
        return {
            "message": self.message.to_graph(),
            "saveToSentItems": self.save_to_sent_items,
        }
