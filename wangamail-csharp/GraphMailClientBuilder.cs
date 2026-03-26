namespace WangaMail.CSharp;

public sealed class GraphMailClientBuilder
{
    private const string DefaultScope = "https://graph.microsoft.com/.default";
    private const string DefaultGraphBase = "https://graph.microsoft.com/v1.0";

    private string? _tenantId;
    private string? _clientId;
    private string? _clientSecret;
    private string? _tokenUrl;
    private string? _graphBase;
    private string? _scope;
    private HttpClient? _httpClient;

    public GraphMailClientBuilder TenantId(string tenantId)
    {
        _tenantId = tenantId;
        return this;
    }

    public GraphMailClientBuilder ClientId(string clientId)
    {
        _clientId = clientId;
        return this;
    }

    public GraphMailClientBuilder ClientSecret(string clientSecret)
    {
        _clientSecret = clientSecret;
        return this;
    }

    public GraphMailClientBuilder TokenUrl(string tokenUrl)
    {
        _tokenUrl = tokenUrl;
        return this;
    }

    public GraphMailClientBuilder GraphBase(string graphBase)
    {
        _graphBase = graphBase;
        return this;
    }

    public GraphMailClientBuilder Scope(string scope)
    {
        _scope = scope;
        return this;
    }

    public GraphMailClientBuilder HttpClient(HttpClient httpClient)
    {
        _httpClient = httpClient;
        return this;
    }

    public GraphMailClient Build()
    {
        if (string.IsNullOrWhiteSpace(_tenantId))
        {
            throw new WangaMailException("Invalid configuration: tenant_id is required.");
        }

        if (string.IsNullOrWhiteSpace(_clientId))
        {
            throw new WangaMailException("Invalid configuration: client_id is required.");
        }

        if (string.IsNullOrWhiteSpace(_clientSecret))
        {
            throw new WangaMailException("Invalid configuration: client_secret is required.");
        }

        var tokenUrl = _tokenUrl ?? $"https://login.microsoftonline.com/{_tenantId}/oauth2/v2.0/token";
        var graphBase = _graphBase ?? DefaultGraphBase;
        var scope = _scope ?? DefaultScope;

        var httpClient = _httpClient ?? new HttpClient();
        var tokenProvider = new TokenProvider(httpClient, _clientId, _clientSecret, tokenUrl, scope);

        return new GraphMailClient(httpClient, tokenProvider, graphBase);
    }
}
