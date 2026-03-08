// Integration test: requires env vars AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET
// and a valid from_user with mailbox. Skip if credentials not set.

use wangamail_rs::{BodyType, GraphMailClient, Message, MessageBody, Recipient, SendMailRequest};

#[tokio::test]
#[ignore] // run with: cargo test --test send_mail_integration -- --ignored
async fn send_mail_e2e() {
    let tenant_id = std::env::var("AZURE_TENANT_ID").ok();
    let client_id = std::env::var("AZURE_CLIENT_ID").ok();
    let client_secret = std::env::var("AZURE_CLIENT_SECRET").ok();
    let from_user = std::env::var("MSGRAPH_FROM_USER").ok();
    let to_email = std::env::var("MSGRAPH_TO_EMAIL").unwrap_or_else(|_| "test@example.com".into());

    let (tenant_id, client_id, client_secret, from_user) = match (
        tenant_id,
        client_id,
        client_secret,
        from_user,
    ) {
        (Some(t), Some(c), Some(s), Some(f)) => (t, c, s, f),
        _ => {
            eprintln!("Skipping: set AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET, MSGRAPH_FROM_USER");
            return;
        }
    };

    let client = GraphMailClient::builder()
        .tenant_id(tenant_id)
        .client_id(client_id)
        .client_secret(client_secret)
        .build()
        .expect("build client");

    let request = SendMailRequest::new(Message {
        subject: "wangamail-rs integration test".to_string(),
        body: MessageBody {
            content_type: BodyType::Text,
            content: "Sent by wangamail-rs integration test.".to_string(),
        },
        to_recipients: vec![Recipient::new(to_email)],
        ..Default::default()
    });

    client
        .send_mail(&from_user, request)
        .await
        .expect("send_mail");
}
