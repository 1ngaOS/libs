from wangamail_py import Message, MessageBody, Recipient, SendMailRequest


def test_send_mail_payload_shape() -> None:
    payload = SendMailRequest(
        message=Message(
            subject="Hello",
            body=MessageBody(content_type="Text", content="Hi"),
            to_recipients=[Recipient.from_email("to@example.com")],
            cc_recipients=[Recipient.from_email("cc@example.com")],
            bcc_recipients=[Recipient.from_email("bcc@example.com")],
        )
    ).to_graph()

    assert payload["saveToSentItems"] is True
    assert payload["message"]["subject"] == "Hello"
    assert payload["message"]["body"]["contentType"] == "Text"
    assert payload["message"]["toRecipients"][0]["emailAddress"]["address"] == "to@example.com"
