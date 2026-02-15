from __future__ import annotations

from typing import Union

from notifykit._notifykit_lib import (
    AccessEvent,
    CreateEvent,
    ModifyDataEvent,
    ModifyMetadataEvent,
    ModifyOtherEvent,
    ModifyUnknownEvent,
    DeleteEvent,
    RenameEvent,
)

Event = Union[
    AccessEvent,
    CreateEvent,
    ModifyDataEvent,
    ModifyMetadataEvent,
    ModifyOtherEvent,
    ModifyUnknownEvent,
    DeleteEvent,
    RenameEvent,
]
