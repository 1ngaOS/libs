"""Custom exceptions for wangamail-py."""


class WangaMailError(Exception):
    """Base exception for wangamail-py."""


class ConfigError(WangaMailError):
    """Raised when client configuration is invalid or missing."""


class GraphAPIError(WangaMailError):
    """Raised when Graph API returns a non-successful response."""
