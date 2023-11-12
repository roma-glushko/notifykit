use crate::events::base::{Event, ObjectType};
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

#[pymethods]
impl CreateEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf, file_type: ObjectType) -> Self {
        Self {
            detected_at_ns,
            path,
            file_type,
        }
    }
}

pub fn from_create_kind(detected_at_ns: u128, path: PathBuf, file_type: CreateKind) -> CreateEvent {
    CreateEvent {
        detected_at_ns,
        path,
        file_type: ObjectType::from(file_type),
    }
}

impl Event for CreateEvent {}
