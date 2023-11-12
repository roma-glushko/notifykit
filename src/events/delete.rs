use crate::events::base::ObjectType;
use notify::event::RemoveKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug)]
pub struct DeleteEvent {
    detected_at_ns: u128,
    path: PathBuf,
    file_type: ObjectType,
}

impl DeleteEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf, file_type: RemoveKind) -> Self {
        Self {
            detected_at_ns,
            path,
            file_type: ObjectType::from(file_type),
        }
    }
}
