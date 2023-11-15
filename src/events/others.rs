use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub struct OtherEvent {
    #[pyo3(get)]
    pub detected_at_ns: u128,
    #[pyo3(get)]
    pub path: PathBuf,
}

#[pymethods]
impl OtherEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct UnknownEvent {
    #[pyo3(get)]
    pub detected_at_ns: u128,
    #[pyo3(get)]
    pub path: PathBuf,
}

#[pymethods]
impl UnknownEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}
