use notify::event::{AccessMode as NotifyAccessMode, AccessKind};
use pyo3::prelude::*;

use crate::events::base::EventAttributes;

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
