import asyncio
from os import PathLike
from typing import Sequence, Protocol, Optional, Any, List
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

Event = AccessEvent | CreateEvent | ModifyDataEvent | ModifyMetadataEvent | ModifyOtherEvent | DeleteEvent | RenameEvent


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

    def __init__(self, debounce_ms: int, debug: bool = False) -> None:
        self._debug = debug

        self._watcher = WatcherWrapper(debounce_ms, debug)

    def watch(
        self, paths: Sequence[PathLike[str]], recursive: bool = True, ignore_permission_errors: bool = False
    ) -> None:
        self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    def unwatch(self, paths: Sequence[str]) -> None:
        self._watcher.unwatch(list(paths))

    def get(self) -> Optional[List[Event]]:
        return self._watcher.get()

    def __aiter__(self) -> "Notifier":
        return self

    def __iter__(self) -> "Notifier":
        return self

    def __next__(self) -> List[Event]:
        event = self._watcher.get()

        if event is None:
            raise StopIteration

        return event

    async def __anext__(self) -> List[Event]:
        events = await asyncio.to_thread(self._watcher.get)

        if events is None:
            raise StopIteration

        return events
