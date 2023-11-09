from os import PathLike
from typing import Sequence, Protocol

from inotifykit._inotifykit_lib import WatcherWrapper, RawEvent


class NotifierT(Protocol):
    def watch(self, paths: Sequence[PathLike], recursive: bool = True, ignore_permission_errors: bool = False) -> None:
        ...

    def unwatch(self, paths: Sequence[str]) -> None:
        ...

    def __enter__(self) -> 'Notifier':
        ...

    def __exit__(self, *args, **kwargs) -> None:
        ...

    def __aiter__(self) -> 'Notifier':
        ...

    def __iter__(self) -> 'Notifier':
        ...


class Notifier:
    """
    Notifier collects filesystem events from the underlying watcher and expose them via sync/async API
    """

    def __init__(self, debug: bool = False, force_polling: bool = False, poll_delay_ms: int = 50) -> None:
        self._debug = debug

        self._watcher = WatcherWrapper(debug, force_polling, poll_delay_ms)

    def watch(self, paths: Sequence[PathLike], recursive: bool = True, ignore_permission_errors: bool = False) -> None:
        self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    def unwatch(self, paths: Sequence[str]) -> None:
        self._watcher.unwatch(list(paths))

    def get(self) -> RawEvent:
        return self._watcher.get()

    def __enter__(self) -> 'Notifier':
        self._watcher.start()

        return self

    def __exit__(self, *args, **kwargs) -> None:
        self._watcher.stop()

    def __del__(self) -> None:
        self._watcher.stop()

    def __aiter__(self) -> 'Notifier':
        return self

    def __iter__(self) -> 'Notifier':
        return self

    def __next__(self) -> RawEvent:
        event = self._watcher.get()

        if event is None:
            raise StopIteration

        return event
