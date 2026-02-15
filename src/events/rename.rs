use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass(from_py_object)]
#[derive(Debug, Clone)]
pub struct RenameEvent {
    #[pyo3(get)]
    pub old_path: PathBuf,
    #[pyo3(get)]
    pub new_path: PathBuf,
}

#[pymethods]
impl RenameEvent {
    #[classattr]
    #[allow(non_upper_case_globals)]
    const __match_args__: (&'static str, &'static str) = ("old_path", "new_path");

    #[new]
    pub fn new(old_path: PathBuf, new_path: PathBuf) -> Self {
        Self { old_path, new_path }
    }

    fn __repr__(&self) -> String {
        format!("RenameEvent({:?}, {:?})", self.old_path, self.new_path,)
    }
}

pub fn from_rename_mode(old_path: PathBuf, new_path: PathBuf) -> RenameEvent {
    RenameEvent { old_path, new_path }
}
