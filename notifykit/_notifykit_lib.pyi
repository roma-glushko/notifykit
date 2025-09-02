from enum import IntEnum
from typing import List, Optional, Any
from notifykit._typing import Event

"""
The lib version
"""
__version__: str

class WatcherError(Exception):
    """Watcher Runtime Error"""

# Main Event Groups

class ObjectType(IntEnum):
    UNKNOWN = 0
    FILE = 1
    DIR = 2
    OTHER = 3

class AccessType(IntEnum):
    UNKNOWN = 0
    READ = 1
    OPEN = 2
    CLOSE = 3
    OTHER = 4

class AccessMode(IntEnum):
    UNKNOWN = 0
    READ = 1
    WRITE = 2
    EXECUTE = 3
    OTHER = 4

class DataType(IntEnum):
    UNKNOWN = 0
    CONTENT = 1
    SIZE = 2
    OTHER = 3

class MetadataType(IntEnum):
    UNKNOWN = 0
    ACCESS_TIME = 1
    WRITE_TIME = 2
    OWNERSHIP = 3
    PERMISSIONS = 4
    EXTENDED = 5
    OTHER = 6

class AccessEvent:
    path: str
    access_type: AccessType
    access_mode: Optional[AccessMode]

    def __init__(self, path: str, access_type: AccessType, access_mode: Optional[AccessMode]) -> None: ...

class CreateEvent:
    path: str
    file_type: ObjectType

    def __init__(self, path: str, file_type: ObjectType) -> None: ...

class ModifyDataEvent:
    path: str
    data_type: DataType

    def __init__(self, path: str, data_type: DataType) -> None: ...

class ModifyMetadataEvent:
    path: str
    metadata_type: MetadataType

    def __init__(self, path: str, metadata_type: MetadataType) -> None: ...

class ModifyOtherEvent:
    path: str

    def __init__(self, path: str) -> None: ...

class ModifyUnknownEvent:
    path: str

    def __init__(self, path: str) -> None: ...

class DeleteEvent:
    path: str
    file_type: ObjectType

    def __init__(self, path: str, file_type: ObjectType) -> None: ...

class RenameEvent:
    old_path: str
    new_path: str

    def __init__(self, old_path: str, new_path: str) -> None: ...

class EventBatchIter:
    def __aiter__(self) -> "EventBatchIter": ...
    async def __anext__(self) -> List[Event]: ...

class WatcherWrapper:
    """
    Watcher listens to filesystem events and retrieves them for the Notifier.
    By default, it tries to pick the most appropriate watching strategy that depends on your OS.
    If that's failed for some reason, it will try to fall back polling filesystem.
    """

    def __init__(
        self,
        debounce_ms: int,
        debug: bool = False,
    ) -> None: ...
    async def watch(self, paths: List[str], recursive: bool = True, ignore_permission_errors: bool = False) -> None: ...
    async def unwatch(self, paths: List[str]) -> None: ...
    def events(self, tick_ms: int) -> EventBatchIter: ...
    def stop(self) -> None: ...