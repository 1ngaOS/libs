# wangapayfast-csharp

Helpers for working with [PayFast](https://www.payfast.co.za/) in .NET services:
checkout payload/signature building, ITN parsing/verification, post-back validation,
and onsite helpers.

## Features

- Build once-off checkout payloads
- Build subscription checkout payloads
- Build custom checkout payloads from arbitrary parameters
- Generate PayFast-compatible checkout signatures
- Parse ITN (`application/x-www-form-urlencoded`) payloads
- Verify ITN signatures
- Validate ITN post-back against PayFast (`/eng/query/validate`)
- Generate onsite payment UUIDs (`/onsite/process`)
- Build recurring card-update URLs
- Sandbox and live environment support

## Install

```bash
dotnet add package wangapayfast-csharp
```

## Usage

```csharp
using WangaPayFast.CSharp;

var client = PayFastClient.Builder()
    .MerchantId(Environment.GetEnvironmentVariable("PAYFAST_MERCHANT_ID")!)
    .MerchantKey(Environment.GetEnvironmentVariable("PAYFAST_MERCHANT_KEY")!)
    .Passphrase(Environment.GetEnvironmentVariable("PAYFAST_PASSPHRASE")!)
    .Environment(PayFastEnvironment.Sandbox)
    .Build();

var checkout = client.BuildOnceOffCheckout(new OnceOffPaymentRequest
{
    PaymentId = "ORDER-1001",
    Amount = "100.00",
    ItemName = "Starter Plan",
    ReturnUrl = "https://example.com/payments/success",
    CancelUrl = "https://example.com/payments/cancel",
    NotifyUrl = "https://example.com/payments/itn"
});

// checkout.Url => https://sandbox.payfast.co.za/eng/process
// checkout.Params => includes merchant fields + signature
```

## ITN + HTTP helpers

```csharp
using WangaPayFast.CSharp;

// Parse incoming ITN body
var itn = ItnNotification.FromBody(rawBodyString);

// Verify signature
var signatureOk = PayFastClient.VerifyItnSignature(
    itn,
    Environment.GetEnvironmentVariable("PAYFAST_PASSPHRASE")
);

// Optional post-back validation
var isValid = await PayFastHttpHelpers.PostBackValidateItnAsync(
    httpClient,
    PayFastEnvironment.Sandbox,
    rawBodyString
);
```

## License

MIT
