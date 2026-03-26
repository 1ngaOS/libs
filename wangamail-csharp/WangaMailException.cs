namespace WangaMail.CSharp;

public sealed class WangaMailException : Exception
{
    public WangaMailException(string message) : base(message)
    {
    }

    public WangaMailException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
