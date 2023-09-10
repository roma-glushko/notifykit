from types import TracebackType
from typing import List

"""
The lib version
"""
__version__: str

class WatcherError(Exception):
    """Watcher Runtime Error"""

class Watcher:
    """
    Watcher listens to filesystem events and retrieves them for the Notifier.
    By default, it tries to pick the most appropriate watching strategy that depends on your OS.
    If that's failed for some reason, it will try to fall back polling filesystem.
    """

    def __init__(
        self,
        debug: bool = False,
        force_polling: bool = False,
        poll_delay_ms: int = 50,
    ) -> None:
        """

        """
        ...

    def watch(self, paths: List[str], recursive: bool = True, ignore_permission_errors: bool = False) -> None:
        """

        """
        ...

    def unwatch(self, paths: List[str]) -> None:
        """
        """
        ...

    def get(self) -> None:
        ...

    def close(self) -> None:
        """
        """
        ...

    def __enter__(self) -> None:
        ...

    def __exit__(self, exc_type: type[BaseException] | None, exc_val: BaseException | None, exc_tb: TracebackType | None) -> None:
        ...
