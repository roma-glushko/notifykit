use crate::events::base::ObjectType;
use notify::event::CreateKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug)]
pub(crate) struct CreateEvent {
    detected_at_ns: u128,
    path: PathBuf,
    file_type: ObjectType,
}

impl CreateEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf, file_type: CreateKind) -> Self {
        Self {
            detected_at_ns,
            path,
            file_type: ObjectType::from(file_type),
        }
    }
}
