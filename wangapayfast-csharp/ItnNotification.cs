using System.Collections.ObjectModel;

namespace WangaPayFast.CSharp;

public sealed class ItnNotification
{
    public IReadOnlyDictionary<string, string> Raw { get; }

    private ItnNotification(SortedDictionary<string, string> raw)
    {
        Raw = new ReadOnlyDictionary<string, string>(raw);
    }

    public static ItnNotification FromBody(byte[] body)
    {
        ArgumentNullException.ThrowIfNull(body);
        return FromBody(System.Text.Encoding.UTF8.GetString(body));
    }

    public static ItnNotification FromBody(string body)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(body);

        var parsed = new SortedDictionary<string, string>(StringComparer.Ordinal);
        var pairs = body.Split('&', StringSplitOptions.RemoveEmptyEntries);
        foreach (var pair in pairs)
        {
            var index = pair.IndexOf('=');
            if (index < 0)
            {
                continue;
            }

            var key = UrlDecode(pair[..index]);
            var value = UrlDecode(pair[(index + 1)..]);
            if (!string.IsNullOrWhiteSpace(key))
            {
                parsed[key] = value;
            }
        }

        return new ItnNotification(parsed);
    }

    public string? Signature =>
        Raw.TryGetValue("signature", out var value) ? value : null;

    public string? PaymentStatusRaw =>
        Raw.TryGetValue("payment_status", out var value) ? value : null;

    public ItnPaymentStatus PaymentStatus()
    {
        return PaymentStatusRaw?.ToUpperInvariant() switch
        {
            "COMPLETE" => ItnPaymentStatus.Complete,
            "FAILED" => ItnPaymentStatus.Failed,
            "CANCELLED" => ItnPaymentStatus.Cancelled,
            _ => ItnPaymentStatus.Unknown
        };
    }

    public bool IsGrossAmount(string expectedAmount)
    {
        if (Raw.TryGetValue("amount_gross", out var amount))
        {
            return string.Equals(amount, expectedAmount, StringComparison.Ordinal);
        }

        return false;
    }

    private static string UrlDecode(string value)
    {
        return Uri.UnescapeDataString(value.Replace("+", " ", StringComparison.Ordinal));
    }
}
