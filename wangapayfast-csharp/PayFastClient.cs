using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace WangaPayFast.CSharp;

public sealed class PayFastClient
{
    private readonly PayFastConfig _config;
    private readonly PayFastEnvironment _environment;

    internal PayFastClient(PayFastConfig config, PayFastEnvironment environment)
    {
        _config = config;
        _environment = environment;
    }

    public static PayFastClientBuilder Builder() => new();

    public CheckoutResponse BuildOnceOffCheckout(OnceOffPaymentRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);
        ValidateOnceOff(request);

        var parameters = BuildParamsFromOnceOff(request);
        AddSignature(parameters);
        return new CheckoutResponse { Url = ResolveProcessUrl(), Params = parameters };
    }

    public CheckoutResponse BuildSubscriptionCheckout(AdvancedPaymentRequest request)
    {
        ArgumentNullException.ThrowIfNull(request);
        ValidateOnceOff(request.Base);

        var parameters = BuildParamsFromOnceOff(request.Base);
        ApplySubscription(parameters, request.Subscription);
        ApplySplit(parameters, request.Split);
        AddSignature(parameters);
        return new CheckoutResponse { Url = ResolveProcessUrl(), Params = parameters };
    }

    public CheckoutResponse BuildCustomCheckout(IDictionary<string, string> parameters)
    {
        ArgumentNullException.ThrowIfNull(parameters);

        var normalized = new SortedDictionary<string, string>(parameters, StringComparer.Ordinal);
        normalized["merchant_id"] = _config.MerchantId!;
        normalized["merchant_key"] = _config.MerchantKey!;
        AddSignature(normalized);
        return new CheckoutResponse { Url = ResolveProcessUrl(), Params = normalized };
    }

    private void ValidateOnceOff(OnceOffPaymentRequest request)
    {
        if (string.IsNullOrWhiteSpace(request.PaymentId))
        {
            throw new WangaPayFastException("Validation error: payment_id is required.");
        }

        if (string.IsNullOrWhiteSpace(request.Amount))
        {
            throw new WangaPayFastException("Validation error: amount is required.");
        }

        if (string.IsNullOrWhiteSpace(request.ItemName))
        {
            throw new WangaPayFastException("Validation error: item_name is required.");
        }
    }

    private SortedDictionary<string, string> BuildParamsFromOnceOff(OnceOffPaymentRequest request)
    {
        var parameters = new SortedDictionary<string, string>(StringComparer.Ordinal)
        {
            ["merchant_id"] = _config.MerchantId!,
            ["merchant_key"] = _config.MerchantKey!,
            ["m_payment_id"] = request.PaymentId,
            ["amount"] = request.Amount,
            ["item_name"] = request.ItemName
        };

        AddIfSet(parameters, "item_description", request.ItemDescription);
        AddIfSet(parameters, "currency", request.Currency ?? request.CurrencyCode);
        AddIfSet(parameters, "name_first", request.NameFirst);
        AddIfSet(parameters, "name_last", request.NameLast);
        AddIfSet(parameters, "email_address", request.EmailAddress);
        AddIfSet(parameters, "cell_number", request.CellNumber);
        AddIfSet(parameters, "return_url", request.ReturnUrl);
        AddIfSet(parameters, "cancel_url", request.CancelUrl);
        AddIfSet(parameters, "notify_url", request.NotifyUrl);
        AddIfSet(parameters, "notify_method", request.NotifyMethod);
        AddIfSet(parameters, "fica_id", request.FicaId);
        AddIfSet(parameters, "confirmation_address", request.ConfirmationAddress);

        if (request.EmailConfirmation.HasValue)
        {
            parameters["email_confirmation"] = request.EmailConfirmation.Value ? "1" : "0";
        }

        foreach (var (key, value) in request.Custom)
        {
            AddIfSet(parameters, key, value);
        }

        return parameters;
    }

    private static void ApplySubscription(SortedDictionary<string, string> parameters, SubscriptionOptions options)
    {
        AddIfSet(parameters, "subscription_type", options.SubscriptionType);
        AddIfSet(parameters, "billing_date", options.BillingDate);
        AddIfSet(parameters, "recurring_amount", options.RecurringAmount);
        AddIfSet(parameters, "frequency", options.Frequency);
        AddIfSet(parameters, "cycles", options.Cycles);

        if (options.SubscriptionNotifyEmail.HasValue)
        {
            parameters["subscription_notify_email"] = options.SubscriptionNotifyEmail.Value ? "true" : "false";
        }

        if (options.SubscriptionNotifyWebhook.HasValue)
        {
            parameters["subscription_notify_webhook"] = options.SubscriptionNotifyWebhook.Value ? "true" : "false";
        }

        if (options.SubscriptionNotifyBuyer.HasValue)
        {
            parameters["subscription_notify_buyer"] = options.SubscriptionNotifyBuyer.Value ? "true" : "false";
        }
    }

    private static void ApplySplit(SortedDictionary<string, string> parameters, SplitPayment split)
    {
        AddIfSet(parameters, "custom_str3", split.PrimaryReceiver);
        AddIfSet(parameters, "custom_str4", split.SecondaryReceiver);
        AddIfSet(parameters, "custom_str5", split.SecondaryAmount);

        var setup = split.Setup;
        if (string.IsNullOrWhiteSpace(setup) && split.SetupPayload is not null)
        {
            setup = JsonSerializer.Serialize(split.SetupPayload);
        }

        AddIfSet(parameters, "setup", setup);

        foreach (var (key, value) in split.Custom)
        {
            AddIfSet(parameters, key, value);
        }
    }

    private void AddSignature(SortedDictionary<string, string> parameters)
    {
        parameters["signature"] = GenerateCheckoutSignature(parameters, _config.Passphrase);
    }

    private static void AddIfSet(IDictionary<string, string> parameters, string key, string? value)
    {
        if (!string.IsNullOrWhiteSpace(value))
        {
            parameters[key] = value;
        }
    }

    private string ResolveProcessUrl()
    {
        return _environment == PayFastEnvironment.Sandbox
            ? "https://sandbox.payfast.co.za/eng/process"
            : "https://www.payfast.co.za/eng/process";
    }

    internal static string GenerateCheckoutSignature(
        IReadOnlyDictionary<string, string> parameters,
        string? passphrase = null)
    {
        var payload = BuildSignaturePayload(parameters, passphrase);
        var bytes = Encoding.UTF8.GetBytes(payload);
        var hash = MD5.HashData(bytes);
        return Convert.ToHexStringLower(hash);
    }

    internal static string BuildSignaturePayload(
        IReadOnlyDictionary<string, string> parameters,
        string? passphrase = null)
    {
        return BuildSignaturePayload(parameters, passphrase, excludeSetupField: true);
    }

    internal static string BuildItnSignaturePayload(
        IReadOnlyDictionary<string, string> parameters,
        string? passphrase = null)
    {
        return BuildSignaturePayload(parameters, passphrase, excludeSetupField: false);
    }

    public static string GenerateItnSignature(
        IReadOnlyDictionary<string, string> parameters,
        string? passphrase = null)
    {
        var payload = BuildItnSignaturePayload(parameters, passphrase);
        var bytes = Encoding.UTF8.GetBytes(payload);
        var hash = MD5.HashData(bytes);
        return Convert.ToHexStringLower(hash);
    }

    public static bool VerifyItnSignature(
        ItnNotification itn,
        string? passphrase = null)
    {
        ArgumentNullException.ThrowIfNull(itn);
        if (string.IsNullOrWhiteSpace(itn.Signature))
        {
            return false;
        }

        var expected = GenerateItnSignature(itn.Raw, passphrase);
        return string.Equals(expected, itn.Signature, StringComparison.OrdinalIgnoreCase);
    }

    private static string BuildSignaturePayload(
        IReadOnlyDictionary<string, string> parameters,
        string? passphrase,
        bool excludeSetupField)
    {
        var filtered = parameters
            .Where(pair => !string.Equals(pair.Key, "signature", StringComparison.OrdinalIgnoreCase))
            .Where(pair => !string.IsNullOrWhiteSpace(pair.Value))
            .Where(pair => !excludeSetupField || !string.Equals(pair.Key, "setup", StringComparison.Ordinal))
            .OrderBy(pair => pair.Key, StringComparer.Ordinal)
            .Select(pair => $"{pair.Key}={Uri.EscapeDataString(pair.Value).Replace("%20", "+")}");

        var payload = string.Join("&", filtered);
        if (!string.IsNullOrWhiteSpace(passphrase))
        {
            payload += $"&passphrase={Uri.EscapeDataString(passphrase).Replace("%20", "+")}";
        }

        return payload;
    }
}
