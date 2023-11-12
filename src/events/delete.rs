use crate::events::base::{Event, ObjectType};
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

#[pymethods]
impl DeleteEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf, file_type: ObjectType) -> Self {
        Self {
            detected_at_ns,
            path,
            file_type,
        }
    }
}

pub fn from_delete_kind(detected_at_ns: u128, path: PathBuf, file_type: RemoveKind) -> DeleteEvent {
    DeleteEvent {
        detected_at_ns,
        path,
        file_type: ObjectType::from(file_type),
    }
}

impl Event for DeleteEvent {}
