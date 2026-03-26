using WangaPayFast.CSharp;

namespace WangaPayFast.CSharp.Tests;

public sealed class PayFastClientTests
{
    [Fact]
    public void Builder_MissingRequiredFields_Throws()
    {
        var ex = Assert.Throws<WangaPayFastException>(() => PayFastClient.Builder().Build());
        Assert.Contains("merchant_id is required", ex.Message);
    }

    [Fact]
    public void BuildOnceOffCheckout_AddsMerchantFieldsAndSignature()
    {
        var client = PayFastClient.Builder()
            .MerchantId("10000100")
            .MerchantKey("46f0cd694581a")
            .Passphrase("abc123")
            .Environment(PayFastEnvironment.Sandbox)
            .Build();

        var response = client.BuildOnceOffCheckout(new OnceOffPaymentRequest
        {
            PaymentId = "ORDER-1",
            Amount = "100.00",
            ItemName = "Subscription",
            ReturnUrl = "https://example.com/success",
            CancelUrl = "https://example.com/cancel",
            NotifyUrl = "https://example.com/itn"
        });

        Assert.Equal("https://sandbox.payfast.co.za/eng/process", response.Url);
        Assert.Equal("10000100", response.Params["merchant_id"]);
        Assert.Equal("46f0cd694581a", response.Params["merchant_key"]);
        Assert.True(response.Params.ContainsKey("signature"));
        Assert.False(string.IsNullOrWhiteSpace(response.Params["signature"]));
    }

    [Fact]
    public void BuildSignaturePayload_ExcludesSetupAndSignatureFields()
    {
        var payload = PayFastClient.BuildSignaturePayload(
            new Dictionary<string, string>
            {
                ["merchant_id"] = "10000100",
                ["merchant_key"] = "46f0cd694581a",
                ["amount"] = "100.00",
                ["item_name"] = "Hello World",
                ["setup"] = "{\"split_payment\":{\"merchant_id\":1}}",
                ["signature"] = "ignoreme"
            },
            "pass"
        );

        Assert.DoesNotContain("setup=", payload, StringComparison.Ordinal);
        Assert.DoesNotContain("signature=", payload, StringComparison.Ordinal);
        Assert.Contains("item_name=Hello+World", payload, StringComparison.Ordinal);
        Assert.EndsWith("passphrase=pass", payload, StringComparison.Ordinal);
    }
}
