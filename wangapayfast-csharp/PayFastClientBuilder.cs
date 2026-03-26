namespace WangaPayFast.CSharp;

public sealed class PayFastClientBuilder
{
    private readonly PayFastConfig _config = new();
    private PayFastEnvironment _environment = PayFastEnvironment.Live;

    public PayFastClientBuilder MerchantId(string merchantId)
    {
        _config.MerchantId = merchantId;
        return this;
    }

    public PayFastClientBuilder MerchantKey(string merchantKey)
    {
        _config.MerchantKey = merchantKey;
        return this;
    }

    public PayFastClientBuilder Passphrase(string passphrase)
    {
        _config.Passphrase = passphrase;
        return this;
    }

    public PayFastClientBuilder Environment(PayFastEnvironment environment)
    {
        _environment = environment;
        return this;
    }

    public PayFastClient Build()
    {
        if (string.IsNullOrWhiteSpace(_config.MerchantId))
        {
            throw new WangaPayFastException("Invalid configuration: merchant_id is required.");
        }

        if (string.IsNullOrWhiteSpace(_config.MerchantKey))
        {
            throw new WangaPayFastException("Invalid configuration: merchant_key is required.");
        }

        return new PayFastClient(_config, _environment);
    }
}
