use crate::events::base::ObjectType;
use notify::event::RemoveKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct DeleteEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub file_type: ObjectType,
}

#[pymethods]
impl DeleteEvent {
    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("path", "file_type");

    #[new]
    pub fn new(path: PathBuf, file_type: ObjectType) -> Self {
        Self { path, file_type }
    }

    fn __repr__(&self) -> String {
        format!("DeleteEvent({:?}, {:?})", self.path, self.file_type,)
    }
}

pub fn from_delete_kind(path: PathBuf, file_type: RemoveKind) -> DeleteEvent {
    DeleteEvent {
        path,
        file_type: ObjectType::from(file_type),
    }
}
