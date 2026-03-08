//! Types for the Microsoft Graph [sendMail](https://learn.microsoft.com/en-us/graph/api/user-sendmail) API.
//!
//! These types map to the JSON payloads used by `POST /users/{id}/sendMail`.

use serde::{Deserialize, Serialize};

/// Body content type for the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BodyType {
    /// Plain text body.
    Text,
    /// HTML body.
    HTML,
}

/// Email address (address + optional display name).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Email address (e.g. `user@example.com`).
    pub address: String,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl EmailAddress {
    /// Create an address with no display name.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            name: None,
        }
    }
    /// Set the display name and return `self`.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// A single recipient (to, cc, or bcc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    /// The recipient’s email address (and optional name).
    pub email_address: EmailAddress,
}

impl Recipient {
    /// Create a recipient by email address only.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            email_address: EmailAddress::new(address),
        }
    }
    /// Create a recipient with an optional display name.
    pub fn with_name(address: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email_address: EmailAddress::new(address).with_name(name),
        }
    }
}

/// Message body (content type + raw content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    /// Whether the content is plain text or HTML.
    #[serde(rename = "contentType")]
    pub content_type: BodyType,
    /// Raw body content (plain text or HTML string).
    pub content: String,
}

/// File attachment (Graph fileAttachment; content as base64).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    /// OData type; use `#microsoft.graph.fileAttachment`.
    #[serde(rename = "@odata.type")]
    pub odata_type: String,
    /// File name (e.g. `document.pdf`).
    pub name: String,
    /// MIME type (e.g. `application/pdf`).
    #[serde(rename = "contentType")]
    pub content_type: String,
    /// File contents, base64-encoded.
    #[serde(rename = "contentBytes")]
    pub content_bytes: String,
}

impl FileAttachment {
    /// Create a file attachment from a base64-encoded payload.
    pub fn new(
        name: impl Into<String>,
        content_type: impl Into<String>,
        content_bytes_base64: impl Into<String>,
    ) -> Self {
        Self {
            odata_type: "#microsoft.graph.fileAttachment".to_string(),
            name: name.into(),
            content_type: content_type.into(),
            content_bytes: content_bytes_base64.into(),
        }
    }
}

/// Attachment variant for Graph sendMail (file attachments).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@odata.type", rename_all = "camelCase")]
pub enum Attachment {
    /// File attachment (use [`FileAttachment`] for construction).
    #[serde(rename = "#microsoft.graph.fileAttachment")]
    File(FileAttachment),
}

/// Message payload for sendMail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Subject line.
    pub subject: String,
    /// Body (text or HTML).
    pub body: MessageBody,
    /// To recipients.
    #[serde(rename = "toRecipients")]
    pub to_recipients: Vec<Recipient>,
    /// CC recipients (optional; omitted if empty).
    #[serde(rename = "ccRecipients", skip_serializing_if = "Vec::is_empty")]
    pub cc_recipients: Vec<Recipient>,
    /// BCC recipients (optional; omitted if empty).
    #[serde(rename = "bccRecipients", skip_serializing_if = "Vec::is_empty")]
    pub bcc_recipients: Vec<Recipient>,
    /// Optional file attachments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<FileAttachment>>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            subject: String::new(),
            body: MessageBody {
                content_type: BodyType::Text,
                content: String::new(),
            },
            to_recipients: vec![],
            cc_recipients: vec![],
            bcc_recipients: vec![],
            attachments: None,
        }
    }
}

/// Request body for `POST /users/{id}/sendMail`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailRequest {
    /// The message to send.
    pub message: Message,
    /// Whether to save a copy in the sender’s Sent Items. Default when using [`SendMailRequest::new`] is `true`.
    #[serde(rename = "saveToSentItems", skip_serializing_if = "Option::is_none")]
    pub save_to_sent_items: Option<bool>,
}

impl SendMailRequest {
    /// Create a send request for the given message. Sent Items saving defaults to `true`.
    pub fn new(message: Message) -> Self {
        Self {
            message,
            save_to_sent_items: Some(true),
        }
    }
    /// Set whether to save a copy in Sent Items.
    pub fn save_to_sent_items(mut self, save: bool) -> Self {
        self.save_to_sent_items = Some(save);
        self
    }
}
