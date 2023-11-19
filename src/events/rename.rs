use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub struct RenameEvent {
    #[pyo3(get)]
    pub old_path: PathBuf,
    #[pyo3(get)]
    pub new_path: PathBuf,
}

#[pymethods]
impl RenameEvent {
    #[new]
    pub fn new(new_path: PathBuf, old_path: PathBuf) -> Self {
        Self { old_path, new_path }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!(
            "RenameEvent({:?}, {:?})",
            slf.borrow().old_path,
            slf.borrow().new_path,
        ))
    }
}

pub fn from_rename_mode(old_path: PathBuf, new_path: PathBuf) -> RenameEvent {
    RenameEvent { old_path, new_path }
}
