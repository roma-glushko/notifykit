from os import PathLike
from typing import Sequence

from inotify_toolkit._inotify_toolkit_lib import Watcher


class Notifier:
    """
    Notifier collects filesystem events from the underlying watcher and expose them via sync/async API
    """

    def __init__(self, debug: bool = False, force_polling: bool = False, poll_delay_ms: int = 50) -> None:
        self._debug = debug

        self._watcher = Watcher(debug, force_polling, poll_delay_ms)

    def watch(self, paths: Sequence[PathLike], recursive: bool = True, ignore_permission_errors: bool = False) -> None:
        self._watcher.watch([str(path) for path in paths], recursive, ignore_permission_errors)

    def unwatch(self, paths: Sequence[str]) -> None:
        self._watcher.unwatch(list(paths))

    def get(self) -> None:
        # TODO: remove this test method
        self._watcher.get()

    def __enter__(self) -> 'Notifier':
        self._watcher.__enter__()

        return self

    def __exit__(self, *args, **kwargs) -> None:
        self._watcher.__exit__(*args, **kwargs)

    def __del__(self) -> None:
        self._watcher.close()

    def __aiter__(self) -> 'Notifier':
        return self

    def __iter__(self) -> 'Notifier':
        return self

    # async def __anext__(self) -> Event:
    #     ...
    #
    # def __next__(self) -> Event:
    #     ...
