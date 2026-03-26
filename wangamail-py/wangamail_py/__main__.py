"""Small CLI entrypoint for manual checks."""

from .client import GraphMailClient


def main() -> None:
    GraphMailClient.from_env()
    print("wangamail-py client initialized from environment")


if __name__ == "__main__":
    main()
