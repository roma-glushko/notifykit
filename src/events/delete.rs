use crate::events::base::ObjectType;
use notify::event::RemoveKind;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub struct DeleteEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub file_type: ObjectType,
}

#[pymethods]
impl DeleteEvent {
    #[new]
    pub fn new(path: PathBuf, file_type: ObjectType) -> Self {
        Self { path, file_type }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!(
            "DeleteEvent({:?}, {:?})",
            slf.borrow().path,
            slf.borrow().file_type,
        ))
    }
}

pub fn from_delete_kind(path: PathBuf, file_type: RemoveKind) -> DeleteEvent {
    DeleteEvent {
        path,
        file_type: ObjectType::from(file_type),
    }
}
