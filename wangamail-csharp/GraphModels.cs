using System.Text.Json;
using System.Text.Json.Serialization;

namespace WangaMail.CSharp;

public enum BodyType
{
    Text,
    Html
}

public sealed class EmailAddress
{
    [JsonPropertyName("address")]
    public string Address { get; set; } = string.Empty;

    [JsonPropertyName("name")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? Name { get; set; }

    public static EmailAddress Create(string address, string? name = null)
    {
        return new EmailAddress { Address = address, Name = name };
    }
}

public sealed class Recipient
{
    [JsonPropertyName("emailAddress")]
    public EmailAddress EmailAddress { get; set; } = new();

    public static Recipient Create(string address, string? name = null)
    {
        return new Recipient { EmailAddress = EmailAddress.Create(address, name) };
    }
}

public sealed class MessageBody
{
    [JsonPropertyName("contentType")]
    [JsonConverter(typeof(BodyTypeJsonConverter))]
    public BodyType ContentType { get; set; } = BodyType.Text;

    [JsonPropertyName("content")]
    public string Content { get; set; } = string.Empty;
}

public sealed class BodyTypeJsonConverter : JsonConverter<BodyType>
{
    public override BodyType Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        var value = reader.GetString();
        return value?.ToUpperInvariant() switch
        {
            "TEXT" => BodyType.Text,
            "HTML" => BodyType.Html,
            _ => throw new JsonException($"Invalid BodyType value: {value}")
        };
    }

    public override void Write(Utf8JsonWriter writer, BodyType value, JsonSerializerOptions options)
    {
        writer.WriteStringValue(value == BodyType.Html ? "HTML" : "Text");
    }
}

public sealed class FileAttachment
{
    [JsonPropertyName("@odata.type")]
    public string ODataType { get; set; } = "#microsoft.graph.fileAttachment";

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("contentType")]
    public string ContentType { get; set; } = "application/octet-stream";

    [JsonPropertyName("contentBytes")]
    public string ContentBytes { get; set; } = string.Empty;
}

public sealed class Message
{
    [JsonPropertyName("subject")]
    public string Subject { get; set; } = string.Empty;

    [JsonPropertyName("body")]
    public MessageBody Body { get; set; } = new();

    [JsonPropertyName("toRecipients")]
    public List<Recipient> ToRecipients { get; set; } = [];

    [JsonPropertyName("ccRecipients")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<Recipient>? CcRecipients { get; set; }

    [JsonPropertyName("bccRecipients")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<Recipient>? BccRecipients { get; set; }

    [JsonPropertyName("attachments")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public List<FileAttachment>? Attachments { get; set; }
}

public sealed class SendMailRequest
{
    [JsonPropertyName("message")]
    public Message Message { get; set; } = new();

    [JsonPropertyName("saveToSentItems")]
    [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public bool? SaveToSentItems { get; set; } = true;
}
