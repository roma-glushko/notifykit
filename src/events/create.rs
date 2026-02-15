use crate::events::base::ObjectType;
use notify::event::CreateKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct CreateEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub file_type: ObjectType,
}

#[pymethods]
impl CreateEvent {
    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("path", "file_type");

    #[new]
    pub fn new(path: PathBuf, file_type: ObjectType) -> Self {
        Self { path, file_type }
    }

    fn __repr__(&self) -> String {
        format!(
            "CreateEvent({:?}, {:?})",
            self.path,
            self.file_type,
        )
    }
}

pub fn from_create_kind(path: PathBuf, file_type: CreateKind) -> CreateEvent {
    CreateEvent {
        path,
        file_type: ObjectType::from(file_type),
    }
}
