# wangamail-py

Send email on behalf of a Microsoft tenant using [Microsoft Graph API](https://learn.microsoft.com/en-us/graph/overview) and app registration credentials (OAuth2 client credentials flow).

## Installation

```bash
pip install wangamail-py
```

## Usage

```python
from wangamail_py import GraphMailClient, Message, MessageBody, Recipient, SendMailRequest

client = GraphMailClient(
    tenant_id="your-tenant-id",
    client_id="your-client-id",
    client_secret="your-client-secret",
)

request = SendMailRequest(
    message=Message(
        subject="Hello from Graph",
        body=MessageBody(content_type="Text", content="This email was sent via Microsoft Graph."),
        to_recipients=[Recipient.from_email("recipient@example.com")],
    )
)

client.send_mail("user@yourtenant.onmicrosoft.com", request)
```

## Environment helper

You can create a client from environment variables:

- `AZURE_TENANT_ID`
- `AZURE_CLIENT_ID`
- `AZURE_CLIENT_SECRET`

```python
from wangamail_py import GraphMailClient

client = GraphMailClient.from_env()
```
