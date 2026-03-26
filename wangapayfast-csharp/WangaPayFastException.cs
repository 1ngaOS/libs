namespace WangaPayFast.CSharp;

public sealed class WangaPayFastException : Exception
{
    public WangaPayFastException(string message) : base(message)
    {
    }

    public WangaPayFastException(string message, Exception innerException) : base(message, innerException)
    {
    }
}
