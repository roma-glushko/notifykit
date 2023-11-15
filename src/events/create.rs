use crate::events::base::ObjectType;
use notify::event::CreateKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub struct CreateEvent {
    #[pyo3(get)]
    pub detected_at_ns: u128,
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub file_type: ObjectType,
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
