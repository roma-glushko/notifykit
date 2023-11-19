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
    FILE = 0
    DIR = 1
    OTHER = 3

class AccessType(IntEnum):
    Read = 1
    Open = 2
    Close = 3
    Other = 4

class AccessMode(IntEnum):
    Any = 0
    Read = 1
    Write = 2
    Execute = 3
    Other = 4

class DataType(IntEnum):
    Any = 0
    Content = 1
    Size = 2
    Other = 3

class MetadataType(IntEnum):
    AccessTime = 0
    WriteTime = 1
    Ownership = 2
    Permissions = 3
    Extended = 4
    Other = 5
    Any = 6

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
        debounce_tick_rate_ms: Optional[int] = None,
    ) -> None: ...
    def watch(self, paths: List[str], recursive: bool = True, ignore_permission_errors: bool = False) -> None: ...
    def unwatch(self, paths: List[str]) -> None: ...
    def get(self) -> Optional[Any]: ...
    def close(self) -> None: ...
    def start(self) -> None: ...
    def stop(self) -> None: ...
