using System.Net.Http.Json;
using System.Text.Json.Serialization;

namespace WangaMail.CSharp;

internal sealed class TokenProvider
{
    private const int RefreshBufferSeconds = 60;
    private readonly HttpClient _httpClient;
    private readonly string _clientId;
    private readonly string _clientSecret;
    private readonly string _tokenUrl;
    private readonly string _scope;
    private readonly SemaphoreSlim _mutex = new(1, 1);
    private TokenCache? _cached;

    public TokenProvider(
        HttpClient httpClient,
        string clientId,
        string clientSecret,
        string tokenUrl,
        string scope)
    {
        _httpClient = httpClient;
        _clientId = clientId;
        _clientSecret = clientSecret;
        _tokenUrl = tokenUrl;
        _scope = scope;
    }

    public async Task<string> GetTokenAsync(CancellationToken cancellationToken = default)
    {
        if (_cached is not null && _cached.ExpiresAt > DateTimeOffset.UtcNow.AddSeconds(RefreshBufferSeconds))
        {
            return _cached.Token;
        }

        await _mutex.WaitAsync(cancellationToken).ConfigureAwait(false);
        try
        {
            if (_cached is not null && _cached.ExpiresAt > DateTimeOffset.UtcNow.AddSeconds(RefreshBufferSeconds))
            {
                return _cached.Token;
            }

            var form = new Dictionary<string, string>
            {
                ["grant_type"] = "client_credentials",
                ["client_id"] = _clientId,
                ["client_secret"] = _clientSecret,
                ["scope"] = _scope
            };

            using var response = await _httpClient.PostAsync(
                _tokenUrl,
                new FormUrlEncodedContent(form),
                cancellationToken).ConfigureAwait(false);

            var body = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
            if (!response.IsSuccessStatusCode)
            {
                throw new WangaMailException($"Authentication failed: HTTP {(int)response.StatusCode} {body}");
            }

            var payload = await response.Content.ReadFromJsonAsync<TokenResponse>(cancellationToken: cancellationToken)
                .ConfigureAwait(false);
            if (payload is null || string.IsNullOrWhiteSpace(payload.AccessToken))
            {
                throw new WangaMailException("Authentication failed: Invalid token response payload.");
            }

            var expiresAt = DateTimeOffset.UtcNow.AddSeconds(Math.Max(payload.ExpiresIn - RefreshBufferSeconds, 1));
            _cached = new TokenCache(payload.AccessToken, expiresAt);
            return payload.AccessToken;
        }
        finally
        {
            _mutex.Release();
        }
    }

    private sealed record TokenCache(string Token, DateTimeOffset ExpiresAt);

    private sealed class TokenResponse
    {
        [JsonPropertyName("access_token")]
        public string AccessToken { get; set; } = string.Empty;

        [JsonPropertyName("expires_in")]
        public int ExpiresIn { get; set; }
    }
}
