use notify::event::{AccessKind, AccessMode as NotifyAccessMode, CreateKind, DataChange, MetadataKind};
use notify::{Config as NotifyConfig, ErrorKind as NotifyErrorKind, Event as NotifyEvent, EventKind};
use pyo3::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum ObjectType {
    Any = 0,
    File = 1,
    Dir = 2,
    Other = 3,
}

#[derive(Debug)]
pub(crate) enum AccessType {
    Any = 0,
    Read = 1,
    Open = 2,
    Close = 3,
    Other = 4,
}

#[derive(Debug)]
pub(crate) enum AccessMode {
    Any = 0,
    Read = 1,
    Write = 2,
    Execute = 3,
    Other = 4,
}

#[derive(Debug)]
pub(crate) enum MetadataType {
    AccessTime = 0,
    WriteTime = 1,
    Ownership = 2,
    Permissions = 3,
    Extended = 4,
    Other = 5,
    Any = 6,
}

#[derive(Debug)]
pub(crate) enum DataChangeType {
    Any = 0,
    Content = 1,
    Size = 2,
    Other = 3,
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
pub(crate) struct AccessedEvent {
    detected_at_ns: u128,
    path: String,
    access_type: AccessType,
    access_mode: Option<AccessMode>,
    attributes: EventAttributes,
}

impl AccessedEvent {
    pub fn new(detected_at_ns: u128, path: String, access_kind: AccessKind, attributes: EventAttributes) -> Self {
        let access_type = match access_kind {
            AccessKind::Read => AccessType::Read,
            AccessKind::Open(_) => AccessType::Open,
            AccessKind::Close(_) => AccessType::Close,
            AccessKind::Other => AccessType::Other,
            AccessKind::Any => AccessType::Any,
        };

        let map_access = |mode: NotifyAccessMode| -> AccessMode {
            return match mode {
                NotifyAccessMode::Read => AccessMode::Read,
                NotifyAccessMode::Write => AccessMode::Write,
                NotifyAccessMode::Execute => AccessMode::Execute,
                NotifyAccessMode::Other => AccessMode::Other,
                NotifyAccessMode::Any => AccessMode::Any,
            };
        };

        let access_mode: Option<AccessMode> = match access_kind {
            AccessKind::Open(access_mode) => Some(map_access(access_mode)),
            AccessKind::Close(access_mode) => Some(map_access(access_mode)),
            _ => None,
        };

        Self {
            detected_at_ns,
            path,
            access_type,
            access_mode,
            attributes,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct CreatedEvent {
    detected_at_ns: u128,
    path: String,
    file_type: ObjectType,
    attributes: EventAttributes,
}

impl CreatedEvent {
    pub fn new(detected_at_ns: u128, path: String, create_kind: CreateKind, attributes: EventAttributes) -> Self {
        let file_type = match create_kind {
            CreateKind::Any => ObjectType::Any,
            CreateKind::File => ObjectType::File,
            CreateKind::Folder => ObjectType::Dir,
            CreateKind::Other => ObjectType::Other,
        };

        Self {
            detected_at_ns,
            path,
            file_type,
            attributes,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct RemovedEvent {
    detected_at_ns: u128,
    path: String,
    file_type: ObjectType,
    attributes: EventAttributes,
}

impl RemovedEvent {
    pub fn new(detected_at_ns: u128, path: String, create_kind: CreateKind, attributes: EventAttributes) -> Self {
        let file_type = match create_kind {
            CreateKind::Any => ObjectType::Any,
            CreateKind::File => ObjectType::File,
            CreateKind::Folder => ObjectType::Dir,
            CreateKind::Other => ObjectType::Other,
        };

        Self {
            detected_at_ns,
            path,
            file_type,
            attributes,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct DataModifiedEvent {
    detected_at_ns: u128,
    path: String,
    data_type: DataChangeType,
    attributes: EventAttributes,
}

impl DataModifiedEvent {
    pub fn new(detected_at_ns: u128, path: String, data_change_kind: DataChange, attributes: EventAttributes) -> Self {
        let data_type: DataChangeType = match data_change_kind {
            DataChange::Content => DataChangeType::Content,
            DataChange::Size => DataChangeType::Size,
            DataChange::Other => DataChangeType::Other,
            DataChange::Any => DataChangeType::Any,
        };

        Self {
            detected_at_ns,
            path,
            data_type,
            attributes,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct MetadataModifiedEvent {
    detected_at_ns: u128,
    path: String,
    metadata_type: MetadataType,
    attributes: EventAttributes,
}

impl MetadataModifiedEvent {
    pub fn new(
        detected_at_ns: u128,
        path: String,
        metadata_change_kind: MetadataKind,
        attributes: EventAttributes,
    ) -> Self {
        let metadata_type: MetadataType = match metadata_change_kind {
            MetadataKind::AccessTime => MetadataType::AccessTime,
            MetadataKind::WriteTime => MetadataType::WriteTime,
            MetadataKind::Permissions => MetadataType::Permissions,
            MetadataKind::Ownership => MetadataType::Ownership,
            MetadataKind::Extended => MetadataType::Extended,
            MetadataKind::Other => MetadataType::Other,
            MetadataKind::Any => MetadataType::Any,
        };

        Self {
            detected_at_ns,
            path,
            metadata_type,
            attributes,
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct OtherModifiedEvent {
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct RenamedEvent {
    detected_at_ns: u128,
    path: String,
    rename_type: RenameType,
    attributes: EventAttributes,
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct OtherEvent {
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct UnknownEvent {
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
}
