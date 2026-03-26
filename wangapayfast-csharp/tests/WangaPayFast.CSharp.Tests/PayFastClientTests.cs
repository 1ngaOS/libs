using WangaPayFast.CSharp;
using System.Net;
using System.Text;
using Xunit;

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

    [Fact]
    public void VerifyItnSignature_ValidPayload_ReturnsTrue()
    {
        var fields = new Dictionary<string, string>
        {
            ["m_payment_id"] = "ORDER-1",
            ["payment_status"] = "COMPLETE",
            ["amount_gross"] = "100.00"
        };
        var signature = PayFastClient.GenerateItnSignature(fields, "abc123");
        var raw = $"m_payment_id=ORDER-1&payment_status=COMPLETE&amount_gross=100.00&signature={signature}";

        var itn = ItnNotification.FromBody(raw);

        Assert.True(PayFastClient.VerifyItnSignature(itn, "abc123"));
        Assert.Equal(ItnPaymentStatus.Complete, itn.PaymentStatus());
        Assert.True(itn.IsGrossAmount("100.00"));
    }

    [Fact]
    public async Task PostBackValidateItnAsync_ValidResponse_ReturnsTrue()
    {
        var handler = new FakeHandler(_ => new HttpResponseMessage(HttpStatusCode.OK)
        {
            Content = new StringContent("VALID", Encoding.UTF8, "text/plain")
        });
        using var httpClient = new HttpClient(handler);

        var result = await PayFastHttpHelpers.PostBackValidateItnAsync(
            httpClient,
            PayFastEnvironment.Sandbox,
            "m_payment_id=1&signature=abc");

        Assert.True(result);
    }

    [Fact]
    public async Task GeneratePaymentIdentifierAsync_ReturnsUuid()
    {
        var handler = new FakeHandler(req =>
        {
            Assert.Equal(HttpMethod.Post, req.Method);
            Assert.Contains("/onsite/process", req.RequestUri?.AbsoluteUri, StringComparison.Ordinal);
            return new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{\"uuid\":\"abc-123\"}", Encoding.UTF8, "application/json")
            };
        });
        using var httpClient = new HttpClient(handler);

        var uuid = await PayFastHttpHelpers.GeneratePaymentIdentifierAsync(
            httpClient,
            new PayFastConfig
            {
                MerchantId = "10000100",
                MerchantKey = "46f0cd694581a",
                Passphrase = "abc123"
            },
            PayFastEnvironment.Sandbox,
            new Dictionary<string, string>
            {
                ["amount"] = "100.00",
                ["item_name"] = "Test"
            });

        Assert.Equal("abc-123", uuid);
    }

    private sealed class FakeHandler : HttpMessageHandler
    {
        private readonly Func<HttpRequestMessage, HttpResponseMessage> _handler;

        public FakeHandler(Func<HttpRequestMessage, HttpResponseMessage> handler)
        {
            _handler = handler;
        }

        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            return Task.FromResult(_handler(request));
        }
    }
}
