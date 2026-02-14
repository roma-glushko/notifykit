from os import PathLike
import logging
from typing import Sequence, Protocol, Optional, List
from notifykit._notifykit_lib import (
    WatcherWrapper,
    EventBatchIter,
)

from notifykit._typing import Event
from notifykit._filters import EventFilter

logger = logging.getLogger(__name__)


class NotifierT(Protocol):
    async def watch(
        self, paths: Sequence[PathLike[str]], recursive: bool = True, ignore_permission_errors: bool = False
    ) -> None: ...

    async def unwatch(self, paths: Sequence[PathLike[str]]) -> None: ...

    def __aiter__(self) -> "NotifierT": ...

    async def __anext__(self) -> List[Event]: ...

    def stop(self) -> None: ...


class Notifier:
    """
    Notifier collects filesystem events from the underlying watcher and expose them via sync/async API
    """

    def __init__(
        self,
        debounce_ms: int = 200,
        tick_ms: int = 50,
        debug: bool = False,
        filter: Optional[EventFilter] = None,
    ) -> None:
        self._debounce_ms = debounce_ms
        self._tick_ms = tick_ms
        self._debug = debug

        self._watcher = WatcherWrapper(debounce_ms, debug)
        self._filter = filter

        self._aiter: Optional[EventBatchIter] = None  # created lazily from Rust iterator

    async def watch(
        self,
        paths: Sequence[PathLike[str]],
        recursive: bool = True,
        ignore_permission_errors: bool = False,
    ) -> None:
        await self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    async def unwatch(self, paths: Sequence[PathLike[str]]) -> None:
        await self._watcher.unwatch([str(path) for path in paths])

    def __aiter__(self) -> "Notifier":
        # start/attach the async iterator from Rust; safe to do before watch()
        if self._aiter is None:
            self._aiter = self._watcher.events(self._tick_ms).__aiter__()

        return self

    async def __anext__(self) -> List[Event]:
        if self._aiter is None:
            self._aiter = self._watcher.events(self._tick_ms).__aiter__()

        while True:
            batch: List[Event] = await self._aiter.__anext__()

            if self._filter:
                batch = [e for e in batch if not self._filter(e)]

            if batch:
                return batch

    def stop(self) -> None:
        self._watcher.stop()
