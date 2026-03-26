using System.Net;
using System.Net.Http.Json;

namespace WangaMail.CSharp;

public sealed class GraphMailClient
{
    private readonly HttpClient _httpClient;
    private readonly TokenProvider _tokenProvider;
    private readonly string _graphBase;

    internal GraphMailClient(HttpClient httpClient, TokenProvider tokenProvider, string graphBase)
    {
        _httpClient = httpClient;
        _tokenProvider = tokenProvider;
        _graphBase = graphBase.TrimEnd('/');
    }

    public static GraphMailClientBuilder Builder() => new();

    public async Task SendMailAsync(
        string fromUser,
        SendMailRequest request,
        CancellationToken cancellationToken = default)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(fromUser);
        ArgumentNullException.ThrowIfNull(request);

        var token = await _tokenProvider.GetTokenAsync(cancellationToken).ConfigureAwait(false);
        var escapedFromUser = Uri.EscapeDataString(fromUser);
        var url = $"{_graphBase}/users/{escapedFromUser}/sendMail";

        using var message = new HttpRequestMessage(HttpMethod.Post, url)
        {
            Content = JsonContent.Create(request)
        };
        message.Headers.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", token);

        using var response = await _httpClient.SendAsync(message, cancellationToken).ConfigureAwait(false);
        if (response.StatusCode == HttpStatusCode.Accepted)
        {
            return;
        }

        var body = await response.Content.ReadAsStringAsync(cancellationToken).ConfigureAwait(false);
        throw new WangaMailException($"Graph API error: sendMail failed: {(int)response.StatusCode} {body}");
    }
}
