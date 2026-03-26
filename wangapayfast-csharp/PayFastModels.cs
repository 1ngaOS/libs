using System.Text.Json.Serialization;

namespace WangaPayFast.CSharp;

public enum PayFastEnvironment
{
    Live,
    Sandbox
}

public sealed class PayFastConfig
{
    public string? MerchantId { get; set; }
    public string? MerchantKey { get; set; }
    public string? Passphrase { get; set; }
}

public sealed class SplitPaymentSetup
{
    [JsonPropertyName("split_payment")]
    public SplitPaymentRule SplitPayment { get; set; } = new();
}

public sealed class SplitPaymentRule
{
    [JsonPropertyName("merchant_id")]
    public ulong MerchantId { get; set; }

    [JsonPropertyName("amount")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Amount { get; set; }

    [JsonPropertyName("percentage")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Percentage { get; set; }

    [JsonPropertyName("min")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Min { get; set; }

    [JsonPropertyName("max")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public ulong? Max { get; set; }
}

public sealed class OnceOffPaymentRequest
{
    public string PaymentId { get; set; } = string.Empty;
    public string Amount { get; set; } = string.Empty;
    public string ItemName { get; set; } = string.Empty;
    public string? ItemDescription { get; set; }
    public string? Currency { get; set; }
    public string? CurrencyCode { get; set; }
    public string? NameFirst { get; set; }
    public string? NameLast { get; set; }
    public string? EmailAddress { get; set; }
    public string? CellNumber { get; set; }
    public string? ReturnUrl { get; set; }
    public string? CancelUrl { get; set; }
    public string? NotifyUrl { get; set; }
    public string? NotifyMethod { get; set; }
    public string? FicaId { get; set; }
    public bool? EmailConfirmation { get; set; }
    public string? ConfirmationAddress { get; set; }
    public Dictionary<string, string> Custom { get; set; } = new();
}

public sealed class SubscriptionOptions
{
    public string? SubscriptionType { get; set; }
    public string? BillingDate { get; set; }
    public string? RecurringAmount { get; set; }
    public string? Frequency { get; set; }
    public string? Cycles { get; set; }
    public bool? SubscriptionNotifyEmail { get; set; }
    public bool? SubscriptionNotifyWebhook { get; set; }
    public bool? SubscriptionNotifyBuyer { get; set; }
}

public sealed class SplitPayment
{
    public string? PrimaryReceiver { get; set; }
    public string? SecondaryReceiver { get; set; }
    public string? SecondaryAmount { get; set; }
    public string? Setup { get; set; }
    public SplitPaymentSetup? SetupPayload { get; set; }
    public Dictionary<string, string> Custom { get; set; } = new();
}

public sealed class AdvancedPaymentRequest
{
    public OnceOffPaymentRequest Base { get; set; } = new();
    public SubscriptionOptions Subscription { get; set; } = new();
    public SplitPayment Split { get; set; } = new();
}

public sealed class CheckoutResponse
{
    public string Url { get; set; } = string.Empty;
    public SortedDictionary<string, string> Params { get; set; } = new(StringComparer.Ordinal);
}
