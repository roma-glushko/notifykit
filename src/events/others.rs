use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug)]
pub struct OtherEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

impl OtherEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}

#[pyclass]
#[derive(Debug)]
pub struct UnknownEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

impl UnknownEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}
