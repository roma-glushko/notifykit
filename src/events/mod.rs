use std::path::Path;

use pyo3::prelude::*;
use pyo3::ToPyObject;

pub(crate) mod access;
pub(crate) mod base;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod modify;
pub(crate) mod rename;

#[derive(Debug, Clone)]
pub enum EventType {
    Access(access::AccessEvent),
    Create(create::CreateEvent),
    Delete(delete::DeleteEvent),
    ModifyMetadata(modify::ModifyMetadataEvent),
    ModifyData(modify::ModifyDataEvent),
    ModifyUnknown(modify::ModifyUnknownEvent),
    ModifyOther(modify::ModifyOtherEvent),
    Rename(rename::RenameEvent),
}

impl EventType {
    /// Returns the primary path for non-rename events.
    /// For rename events, use the fields directly.
    pub fn path(&self) -> Option<&Path> {
        match self {
            EventType::Access(e) => Some(&e.path),
            EventType::Create(e) => Some(&e.path),
            EventType::Delete(e) => Some(&e.path),
            EventType::ModifyMetadata(e) => Some(&e.path),
            EventType::ModifyData(e) => Some(&e.path),
            EventType::ModifyUnknown(e) => Some(&e.path),
            EventType::ModifyOther(e) => Some(&e.path),
            EventType::Rename(_) => None,
        }
    }
}

impl ToPyObject for EventType {
    fn to_object(&self, py: Python) -> PyObject {
        match self {
            EventType::Access(event) => event.clone().into_py(py),
            EventType::Create(event) => event.clone().into_py(py),
            EventType::Delete(event) => event.clone().into_py(py),
            EventType::ModifyMetadata(event) => event.clone().into_py(py),
            EventType::ModifyData(event) => event.clone().into_py(py),
            EventType::ModifyOther(event) => event.clone().into_py(py),
            EventType::ModifyUnknown(event) => event.clone().into_py(py),
            EventType::Rename(event) => event.clone().into_py(py),
        }
    }
}
