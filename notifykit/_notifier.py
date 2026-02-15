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
        event_buffer_size: int = 1024,
        debug: bool = False,
        filter: Optional[EventFilter] = None,
    ) -> None:
        self._debounce_ms = debounce_ms
        self._tick_ms = tick_ms
        self._debug = debug

        self._watcher = WatcherWrapper(debounce_ms, event_buffer_size, debug)

        # Extract filter config to pass to the Rust side
        if filter is not None:
            self._ignore_dirs = list(filter.ignore_dirs)
            self._ignore_patterns = list(filter.ignore_object_patterns)
            self._ignore_paths = [str(p) for p in filter.ignore_paths]
        else:
            self._ignore_dirs = []
            self._ignore_patterns = []
            self._ignore_paths = []

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
            self._aiter = self._watcher.events(
                self._tick_ms, self._ignore_dirs, self._ignore_patterns, self._ignore_paths,
            ).__aiter__()

        return self

    async def __anext__(self) -> List[Event]:
        if self._aiter is None:
            self._aiter = self._watcher.events(
                self._tick_ms, self._ignore_dirs, self._ignore_patterns, self._ignore_paths,
            ).__aiter__()

        return await self._aiter.__anext__()

    def stop(self) -> None:
        self._watcher.stop()
