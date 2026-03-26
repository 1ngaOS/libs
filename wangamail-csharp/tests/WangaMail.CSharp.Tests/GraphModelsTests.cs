using System.Text.Json;
using WangaMail.CSharp;
using Xunit;

namespace WangaMail.CSharp.Tests;

public sealed class GraphModelsTests
{
    [Fact]
    public void SerializeSendMailRequest_UsesGraphFieldNames()
    {
        var request = new SendMailRequest
        {
            Message = new Message
            {
                Subject = "Test",
                Body = new MessageBody
                {
                    ContentType = BodyType.Text,
                    Content = "Hello"
                },
                ToRecipients = [Recipient.Create("to@example.com")]
            }
        };

        var json = JsonSerializer.Serialize(request);

        Assert.Contains("\"saveToSentItems\":true", json);
        Assert.Contains("\"toRecipients\"", json);
        Assert.Contains("\"emailAddress\"", json);
        Assert.Contains("\"contentType\":\"Text\"", json);
    }

    [Fact]
    public void Builder_MissingRequiredFields_Throws()
    {
        var ex = Assert.Throws<WangaMailException>(() => GraphMailClient.Builder().Build());
        Assert.Contains("tenant_id is required", ex.Message);
    }
}
