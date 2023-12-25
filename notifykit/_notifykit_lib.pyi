from enum import IntEnum
from pathlib import Path
from typing import List, Optional, Any

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
    path: Path
    access_type: AccessType
    access_mode: Optional[AccessMode]

class CreateEvent:
    path: Path
    file_type: ObjectType

class ModifyDataEvent:
    path: Path
    data_type: DataType

class ModifyMetadataEvent:
    path: Path
    metadata_type: MetadataType

class ModifyOtherEvent:
    path: Path

class ModifyUnknownEvent:
    path: Path

class DeleteEvent:
    path: Path
    file_type: ObjectType

class RenameEvent:
    old_path: Path
    new_path: Path

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
    def watch(self, paths: List[str], recursive: bool = True, ignore_permission_errors: bool = False) -> None: ...
    def unwatch(self, paths: List[str]) -> None: ...
    def get(self, tick_ms: int, stop_event: Any) -> Optional[Any]: ...
    def set_stopping(self) -> None: ...
