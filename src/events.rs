use notify::event::{AccessMode as NotifyAccessMode, DataChange, EventAttributes as NotifyEventAttrs, MetadataKind};
use pyo3::prelude::*;

#[derive(Debug)]
pub(crate) enum EventType {
    Create = 0,
    Access = 1,
    Remove = 2,
    Modify = 3,
    Other = 4,
}

#[derive(Debug)]
pub(crate) enum ObjectType {
    File = 0,
    Dir = 1,
    Other = 3,
}

#[derive(Debug)]
pub(crate) enum AccessType {
    Read = 0,
    Open = 1,
    Close = 2,
    Other = 3,
}

#[derive(Debug)]
pub(crate) enum AccessMode {
    Read = 0,
    Write = 1,
    Execute = 2,
    Other = 3,
}

impl AccessMode {
    pub(crate) fn from_raw(raw_mode: NotifyAccessMode) -> Option<Self> {
        return match raw_mode {
            NotifyAccessMode::Read => Some(Self::Read),
            NotifyAccessMode::Write => Some(Self::Write),
            NotifyAccessMode::Execute => Some(Self::Execute),
            NotifyAccessMode::Other => Some(Self::Other),
            NotifyAccessMode::Any => None,
        };
    }
}

#[derive(Debug)]
pub(crate) enum ModifyType {
    Metadata = 0,
    Data = 1,
    Rename = 2,
    Other = 3,
}

#[derive(Debug)]
pub(crate) enum MetadataType {
    AccessTime = 0,
    WriteTime = 1,
    Ownership = 2,
    Permissions = 3,
    Extended = 4,
    Other = 5,
}

impl MetadataType {
    pub(crate) fn from_raw(raw_mode: MetadataKind) -> Option<Self> {
        return match raw_mode {
            MetadataKind::AccessTime => Some(Self::AccessTime),
            MetadataKind::WriteTime => Some(Self::WriteTime),
            MetadataKind::Ownership => Some(Self::Ownership),
            MetadataKind::Permissions => Some(Self::Permissions),
            MetadataKind::Extended => Some(Self::Extended),
            MetadataKind::Other => Some(Self::Other),
            MetadataKind::Any => None,
        };
    }
}

#[derive(Debug)]
pub(crate) enum DataChangeType {
    Content = 0,
    Size = 1,
    Other = 2,
}

impl DataChangeType {
    pub(crate) fn from_raw(data_changed: DataChange) -> Option<Self> {
        return match data_changed {
            DataChange::Content => Some(Self::Content),
            DataChange::Size => Some(Self::Size),
            DataChange::Other => Some(Self::Other),
            DataChange::Any => None,
        };
    }
}

#[derive(Debug)]
pub(crate) enum RenameType {
    From = 0,
    To = 1,
    Both = 2,
    Other = 3,
}

#[pyclass]
#[derive(Clone, Debug)]
pub(crate) struct EventAttributes {
    pub(crate) tracker: Option<usize>,
    // TODO: add the rest of data
}

#[pymethods]
impl EventAttributes {
    // pub(crate) fn from_raw_attrs(attrs: EventAttributes) -> Self {
    //     EventAttributes {
    //         tracker: attrs.tracker(),
    //     }
    // }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct RawEvent {
    pub(crate) event_type: Option<EventType>,

    pub(crate) object_type: Option<ObjectType>,

    pub(crate) access_type: Option<AccessType>,
    pub(crate) access_mode: Option<AccessMode>,

    pub(crate) modify_type: Option<ModifyType>,
    pub(crate) metadata_type: Option<MetadataType>,
    pub(crate) data_change_type: Option<DataChangeType>,
    pub(crate) rename_mode: Option<RenameType>,

    pub(crate) detected_at_ns: u128,
    pub(crate) path: String,
    pub(crate) attributes: EventAttributes,
}

#[pymethods]
impl RawEvent {
    // #[new]
    // pub(crate) fn new(event_type: Option<EventType>, object_type: Option<ObjectType>, access_type: Option<AccessType>, access_mode: Option<AccessMode>, detected_at_ns: u128, path: String, attributes: EventAttributes) -> Self {
    //     RawEvent {
    //         event_type,
    //         object_type,
    //         access_type,
    //         access_mode,
    //         // TODO: allow to pass these values
    //         modify_type: None,
    //         metadata_type: None,
    //         data_change_type: None,
    //         rename_mode: None,
    //         detected_at_ns,
    //         path,
    //         attributes,
    //     }
    // }

    fn __repr__(&self) -> String {
        return format!("RawEvent ({:#?})", self);
    }
}

pub(crate) fn new_access_event(
    access_type: Option<AccessType>,
    access_mode: Option<AccessMode>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Access),
        object_type: None,
        access_type,
        access_mode,
        modify_type: None,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_create_event(
    object_type: Option<ObjectType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Create),
        object_type,
        access_type: None,
        access_mode: None,
        modify_type: None,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_remove_event(
    object_type: Option<ObjectType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Remove),
        object_type,
        access_type: None,
        access_mode: None,
        modify_type: None,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_modify_event(
    modify_type: Option<ModifyType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Modify),
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_modify_metadata_event(
    metadata_type: Option<MetadataType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Modify),
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type: Some(ModifyType::Metadata),
        metadata_type,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_modify_data_event(
    data_change_type: Option<DataChangeType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Modify),
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type: Some(ModifyType::Data),
        metadata_type: None,
        data_change_type,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_rename_event(
    rename_mode: Option<RenameType>,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Modify),
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type: Some(ModifyType::Rename),
        metadata_type: None,
        data_change_type: None,
        rename_mode,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_other_event(detected_at_ns: u128, path: String, attributes: EventAttributes) -> RawEvent {
    RawEvent {
        event_type: Some(EventType::Other),
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type: None,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}

pub(crate) fn new_unknown_event(detected_at_ns: u128, path: String, attributes: EventAttributes) -> RawEvent {
    RawEvent {
        event_type: None,
        object_type: None,
        access_type: None,
        access_mode: None,
        modify_type: None,
        metadata_type: None,
        data_change_type: None,
        rename_mode: None,
        detected_at_ns,
        path,
        attributes,
    }
}
