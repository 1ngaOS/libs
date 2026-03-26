using System.Net.Http.Json;
using System.Text;

namespace WangaPayFast.CSharp;

public static class PayFastHttpHelpers
{
    public static async Task<bool> PostBackValidateItnAsync(
        HttpClient httpClient,
        PayFastEnvironment environment,
        string rawItnBody,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(httpClient);
        ArgumentException.ThrowIfNullOrWhiteSpace(rawItnBody);

        var url = environment == PayFastEnvironment.Sandbox
            ? "https://sandbox.payfast.co.za/eng/query/validate"
            : "https://www.payfast.co.za/eng/query/validate";

        using var content = new StringContent(rawItnBody, Encoding.UTF8, "application/x-www-form-urlencoded");
        using var response = await httpClient.PostAsync(url, content, cancellationToken).ConfigureAwait(false);
        var body = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);

        if (!response.IsSuccessStatusCode)
        {
            throw new WangaPayFastException($"PayFast post-back validation failed: {(int)response.StatusCode} {body}");
        }

        return string.Equals(body.Trim(), "VALID", StringComparison.OrdinalIgnoreCase);
    }

    public static async Task<string> GeneratePaymentIdentifierAsync(
        HttpClient httpClient,
        PayFastConfig config,
        PayFastEnvironment environment,
        IDictionary<string, string> checkoutParams,
        CancellationToken cancellationToken = default)
    {
        ArgumentNullException.ThrowIfNull(httpClient);
        ArgumentNullException.ThrowIfNull(config);
        ArgumentNullException.ThrowIfNull(checkoutParams);

        if (string.IsNullOrWhiteSpace(config.MerchantId))
        {
            throw new WangaPayFastException("Invalid configuration: merchant_id is required.");
        }

        if (string.IsNullOrWhiteSpace(config.MerchantKey))
        {
            throw new WangaPayFastException("Invalid configuration: merchant_key is required.");
        }

        var normalized = new SortedDictionary<string, string>(checkoutParams, StringComparer.Ordinal)
        {
            ["merchant_id"] = config.MerchantId,
            ["merchant_key"] = config.MerchantKey
        };
        normalized["signature"] = PayFastClient.GenerateCheckoutSignature(normalized, config.Passphrase);

        var url = environment == PayFastEnvironment.Sandbox
            ? "https://sandbox.payfast.co.za/onsite/process"
            : "https://www.payfast.co.za/onsite/process";

        using var response = await httpClient.PostAsync(url, new FormUrlEncodedContent(normalized), cancellationToken)
            .ConfigureAwait(false);
        var body = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);

        if (!response.IsSuccessStatusCode)
        {
            throw new WangaPayFastException($"PayFast onsite process failed: {(int)response.StatusCode} {body}");
        }

        var parsed = await response.Content.ReadFromJsonAsync<OnsiteProcessResponse>(cancellationToken: cancellationToken)
            .ConfigureAwait(false);
        if (parsed is null || string.IsNullOrWhiteSpace(parsed.Uuid))
        {
            throw new WangaPayFastException("PayFast onsite process failed: invalid UUID response.");
        }

        return parsed.Uuid;
    }

    public static string CardUpdateUrl(PayFastEnvironment environment, string token, string? returnUrl = null)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(token);

        var baseUrl = environment == PayFastEnvironment.Sandbox
            ? "https://sandbox.payfast.co.za/eng/recurring/update"
            : "https://www.payfast.co.za/eng/recurring/update";

        if (string.IsNullOrWhiteSpace(returnUrl))
        {
            return $"{baseUrl}/{token}";
        }

        var encoded = Uri.EscapeDataString(returnUrl).Replace("%20", "+", StringComparison.Ordinal);
        return $"{baseUrl}/{token}?return={encoded}";
    }
}
