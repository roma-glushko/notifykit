from os import PathLike
from typing import Sequence, List

from notifykit import Event


class NotifierMock:
    """
    A notifier mock that allows to control filesystems events without actually watching the filesystem
    """

    def __init__(self, events_batches: List[List[Event]]) -> None:
        self._watch_paths: List[PathLike[str]] = []
        self._events_batches = events_batches

    @property
    def watch_paths(self) -> List[PathLike[str]]:
        return self._watch_paths

    @property
    def events_batches(self) -> List[List[Event]]:
        return self._events_batches

    def add_event_batch(self, events_batch: List[Event]) -> None:
        self._events_batches.append(events_batch)

    def watch(
        self,
        paths: Sequence[PathLike[str]],
        recursive: bool = True,
        ignore_permission_errors: bool = False,
    ) -> None:
        self._watch_paths.extend(paths)

    def unwatch(self, paths: Sequence[PathLike[str]]) -> None:
        for path in paths:
            self._watch_paths.remove(path)

    def __aiter__(self) -> "NotifierMock":
        return self

    def __iter__(self) -> "NotifierMock":
        return self

    def __next__(self) -> List[Event]:
        if not self._events_batches:
            raise StopIteration

        return self._events_batches.pop(0)

    async def __anext__(self) -> List[Event]:
        if not self._events_batches:
            raise StopAsyncIteration

        return self._events_batches.pop(0)
