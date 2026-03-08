//! Types for Microsoft Graph sendMail API.

use serde::{Deserialize, Serialize};

/// Body content type for the message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BodyType {
    Text,
    HTML,
}

/// Email address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl EmailAddress {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            name: None,
        }
    }
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Recipient (to/cc/bcc).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipient {
    pub email_address: EmailAddress,
}

impl Recipient {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            email_address: EmailAddress::new(address),
        }
    }
    pub fn with_name(address: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email_address: EmailAddress::new(address).with_name(name),
        }
    }
}

/// Message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    #[serde(rename = "contentType")]
    pub content_type: BodyType,
    pub content: String,
}

/// File attachment (base64 content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    #[serde(rename = "@odata.type")]
    pub odata_type: String,
    pub name: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "contentBytes")]
    pub content_bytes: String,
}

impl FileAttachment {
    pub fn new(name: impl Into<String>, content_type: impl Into<String>, content_bytes_base64: impl Into<String>) -> Self {
        Self {
            odata_type: "#microsoft.graph.fileAttachment".to_string(),
            name: name.into(),
            content_type: content_type.into(),
            content_bytes: content_bytes_base64.into(),
        }
    }
}

/// Attachment enum for Graph (only fileAttachment used in sendMail).
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "@odata.type", rename_all = "camelCase")]
pub enum Attachment {
    #[serde(rename = "#microsoft.graph.fileAttachment")]
    File(FileAttachment),
}

/// Message payload for sendMail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub subject: String,
    pub body: MessageBody,
    #[serde(rename = "toRecipients")]
    pub to_recipients: Vec<Recipient>,
    #[serde(rename = "ccRecipients", skip_serializing_if = "Vec::is_empty")]
    pub cc_recipients: Vec<Recipient>,
    #[serde(rename = "bccRecipients", skip_serializing_if = "Vec::is_empty")]
    pub bcc_recipients: Vec<Recipient>,
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

/// Request body for POST /users/{id}/sendMail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMailRequest {
    pub message: Message,
    #[serde(rename = "saveToSentItems", skip_serializing_if = "Option::is_none")]
    pub save_to_sent_items: Option<bool>,
}

impl SendMailRequest {
    pub fn new(message: Message) -> Self {
        Self {
            message,
            save_to_sent_items: Some(true),
        }
    }
    pub fn save_to_sent_items(mut self, save: bool) -> Self {
        self.save_to_sent_items = Some(save);
        self
    }
}
