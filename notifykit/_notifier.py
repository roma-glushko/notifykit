from os import PathLike
import anyio
import logging
from typing import Sequence, Protocol, Optional, List
from notifykit._notifykit_lib import (
    WatcherWrapper,
)

from notifykit._typing import Event
from notifykit._filters import EventFilter

logger = logging.getLogger(__name__)


class AnyEvent(Protocol):
    def is_set(self) -> bool:
        ...

    def set(self) -> None:
        ...


class NotifierT(Protocol):
    def watch(
        self, paths: Sequence[PathLike[str]], recursive: bool = True, ignore_permission_errors: bool = False
    ) -> None:
        ...

    def unwatch(self, paths: Sequence[PathLike[str]]) -> None:
        ...

    def __aiter__(self) -> "Notifier":
        ...

    def __iter__(self) -> "Notifier":
        ...

    def __next__(self) -> List[Event]:
        ...

    async def __anext__(self) -> List[Event]:
        ...


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
        stop_event: Optional[AnyEvent] = None,
    ) -> None:
        self._debounce_ms = debounce_ms
        self._tick_ms = tick_ms
        self._debug = debug

        self._watcher = WatcherWrapper(debounce_ms, debug)
        self._filter = filter
        self._stop_event = stop_event if stop_event else anyio.Event()

    def watch(
        self,
        paths: Sequence[PathLike[str]],
        recursive: bool = True,
        ignore_permission_errors: bool = False,
    ) -> None:
        self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    def unwatch(self, paths: Sequence[str]) -> None:
        self._watcher.unwatch(list(paths))

    def get(self) -> Optional[List[Event]]:
        return self._watcher.get(self._tick_ms, self._stop_event)

    def set_stopping(self) -> None:
        self._stop_event.set()

    def __aiter__(self) -> "Notifier":
        return self

    def __iter__(self) -> "Notifier":
        return self

    def __next__(self) -> List[Event]:
        events = self._watcher.get(self._tick_ms, self._stop_event)

        if events is None:
            raise StopIteration

        if self._filter:
            if self._debug:
                logger.debug(f"events before filtering: {events}")

            events = [event for event in events if not self._filter(event)]

        return events

    async def __anext__(self) -> List[Event]:
        CancelledError = anyio.get_cancelled_exc_class()

        async with anyio.create_task_group() as tg:
            try:
                events = await anyio.to_thread.run_sync(self._watcher.get, self._tick_ms, self._stop_event)
            except (CancelledError, KeyboardInterrupt):
                self._stop_event.set()
                # suppressing KeyboardInterrupt wouldn't stop it getting raised by the top level asyncio.run call
                raise

            tg.cancel_scope.cancel()

            if events is None:
                raise StopAsyncIteration

            if self._filter:
                if self._debug:
                    logger.debug(f"events before filtering: {events}")

                events = [event for event in events if not self._filter(event)]

            return events
