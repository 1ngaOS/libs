# wangapayfast-csharp

Helpers for building [PayFast](https://www.payfast.co.za/) checkout payloads and signatures in .NET services.

## Features

- Build once-off checkout payloads
- Build subscription checkout payloads
- Build custom checkout payloads from arbitrary parameters
- Generate PayFast-compatible checkout signatures
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

## License

MIT
