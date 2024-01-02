from notifykit._notifykit_lib import (
    AccessEvent,
    CreateEvent,
    ModifyDataEvent,
    ModifyMetadataEvent,
    ModifyOtherEvent,
    DeleteEvent,
    RenameEvent,
)

Event = AccessEvent | CreateEvent | ModifyDataEvent | ModifyMetadataEvent | ModifyOtherEvent | DeleteEvent | RenameEvent
