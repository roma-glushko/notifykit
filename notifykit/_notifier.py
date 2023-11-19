from os import PathLike
from typing import Sequence, Protocol, Optional, Any
from notifykit._notifykit_lib import (
    WatcherWrapper,
    AccessEvent,
    CreateEvent,
    ModifyDataEvent,
    ModifyMetadataEvent,
    ModifyOtherEvent,
    DeleteEvent,
    RenameEvent,
)

Events = (
    AccessEvent | CreateEvent | ModifyDataEvent | ModifyMetadataEvent | ModifyOtherEvent | DeleteEvent | RenameEvent
)


class NotifierT(Protocol):
    def watch(
        self, paths: Sequence[PathLike[str]], recursive: bool = True, ignore_permission_errors: bool = False
    ) -> None:
        ...

    def unwatch(self, paths: Sequence[str]) -> None:
        ...

    def __enter__(self) -> "Notifier":
        ...

    def __exit__(self, *args: Any, **kwargs: Any) -> None:
        ...

    def __aiter__(self) -> "Notifier":
        ...

    def __iter__(self) -> "Notifier":
        ...


class Notifier:
    """
    Notifier collects filesystem events from the underlying watcher and expose them via sync/async API
    """

    def __init__(self, debounce_ms: int, debounce_tick_rate_ms: Optional[int] = None, debug: bool = False) -> None:
        self._debug = debug

        self._watcher = WatcherWrapper(debounce_ms, debug, debounce_tick_rate_ms)

    def watch(
        self, paths: Sequence[PathLike[str]], recursive: bool = True, ignore_permission_errors: bool = False
    ) -> None:
        self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    def unwatch(self, paths: Sequence[str]) -> None:
        self._watcher.unwatch(list(paths))

    def get(self) -> Optional[Events]:
        return self._watcher.get()

    def __enter__(self) -> "Notifier":
        self._watcher.start()

        return self

    def __exit__(self, *args: Any, **kwargs: Any) -> None:
        self._watcher.stop()

    def __del__(self) -> None:
        self._watcher.stop()

    def __aiter__(self) -> "Notifier":
        return self

    def __iter__(self) -> "Notifier":
        return self

    def __next__(self) -> Events:
        event = self._watcher.get()

        if event is None:
            raise StopIteration

        return event
