use notify::event::{AccessKind, AccessMode as NotifyAccessMode};
use pyo3::prelude::*;
use std::convert::From;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Copy, Clone)]
pub enum AccessType {
    UNKNOWN = 0,
    READ = 1,
    OPEN = 2,
    CLOSE = 3,
    OTHER = 4,
}

impl From<AccessKind> for AccessType {
    fn from(kind: AccessKind) -> Self {
        match kind {
            AccessKind::Read => AccessType::READ,
            AccessKind::Open(_) => AccessType::OPEN,
            AccessKind::Close(_) => AccessType::CLOSE,
            AccessKind::Other => AccessType::OTHER,
            AccessKind::Any => AccessType::UNKNOWN,
        }
    }
}

#[pyclass]
#[derive(Debug, Copy, Clone)]
pub enum AccessMode {
    UNKNOWN = 0,
    READ = 1,
    WRITE = 2,
    EXECUTE = 3,
    OTHER = 4,
}

impl From<NotifyAccessMode> for AccessMode {
    fn from(kind: NotifyAccessMode) -> Self {
        match kind {
            NotifyAccessMode::Read => AccessMode::READ,
            NotifyAccessMode::Write => AccessMode::WRITE,
            NotifyAccessMode::Execute => AccessMode::EXECUTE,
            NotifyAccessMode::Other => AccessMode::OTHER,
            NotifyAccessMode::Any => AccessMode::UNKNOWN,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct AccessEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub access_type: AccessType,
    #[pyo3(get)]
    pub access_mode: Option<AccessMode>,
}

#[pymethods]
impl AccessEvent {
    #[new]
    pub fn new(path: PathBuf, access_type: AccessType, access_mode: Option<AccessMode>) -> Self {
        Self {
            path,
            access_type,
            access_mode,
        }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!(
            "AccessEvent({:?}, {:?},  {:?})",
            slf.borrow().path,
            slf.borrow().access_type,
            slf.borrow().access_mode,
        ))
    }
}

pub fn from_access_kind(path: PathBuf, access_kind: AccessKind) -> AccessEvent {
    let access_mode: Option<AccessMode> = match access_kind {
        AccessKind::Open(access_mode) => Some(AccessMode::from(access_mode)),
        AccessKind::Close(access_mode) => Some(AccessMode::from(access_mode)),
        _ => None,
    };

    AccessEvent {
        path,
        access_type: AccessType::from(access_kind),
        access_mode,
    }
}
