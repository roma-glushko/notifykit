"""
The lib version
"""
__version__: str

class WatcherError(Exception):
    """Watcher Runtime Error"""

class Watcher:
    def watch(self) -> None:
        ...

    def unwatch(self) -> None:
        ...
