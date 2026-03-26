# wangamail-csharp

Send email on behalf of a Microsoft tenant using Microsoft Graph API and app registration credentials (OAuth2 client credentials flow).

## Features

- Typed client API for `POST /users/{id}/sendMail`
- OAuth2 token acquisition and in-memory token caching
- Strongly typed mail payload models (subject/body/recipients/attachments)
- Sovereign cloud support via `TokenUrl(...)` and `GraphBase(...)`

## Install

Add the package to your project:

```bash
dotnet add package wangamail-csharp
```

## Usage

```csharp
using WangaMail.CSharp;

var client = GraphMailClient.Builder()
    .TenantId(Environment.GetEnvironmentVariable("AZURE_TENANT_ID")!)
    .ClientId(Environment.GetEnvironmentVariable("AZURE_CLIENT_ID")!)
    .ClientSecret(Environment.GetEnvironmentVariable("AZURE_CLIENT_SECRET")!)
    .Build();

var request = new SendMailRequest
{
    Message = new Message
    {
        Subject = "Hello from Graph",
        Body = new MessageBody
        {
            ContentType = BodyType.Text,
            Content = "This email was sent via Microsoft Graph."
        },
        ToRecipients =
        [
            Recipient.Create("recipient@example.com")
        ]
    }
};

await client.SendMailAsync("user@yourtenant.onmicrosoft.com", request);
```

## Azure setup

1. Register an application in Microsoft Entra ID.
2. Create a client secret.
3. Add Microsoft Graph application permission `Mail.Send`.
4. Grant admin consent.

## License

MIT
