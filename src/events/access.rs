use notify::event::{AccessKind, AccessMode as NotifyAccessMode};
use pyo3::prelude::*;
use std::convert::From;
use std::path::PathBuf;

#[pyclass(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Copy, Clone)]
pub enum AccessType {
    Unknown = 0,
    Read = 1,
    Open = 2,
    Close = 3,
    Other = 4,
}

impl From<AccessKind> for AccessType {
    fn from(kind: AccessKind) -> Self {
        match kind {
            AccessKind::Read => AccessType::Read,
            AccessKind::Open(_) => AccessType::Open,
            AccessKind::Close(_) => AccessType::Close,
            AccessKind::Other => AccessType::Other,
            AccessKind::Any => AccessType::Unknown,
        }
    }
}

#[pyclass(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Copy, Clone)]
pub enum AccessMode {
    Unknown = 0,
    Read = 1,
    Write = 2,
    Execute = 3,
    Other = 4,
}

impl From<NotifyAccessMode> for AccessMode {
    fn from(kind: NotifyAccessMode) -> Self {
        match kind {
            NotifyAccessMode::Read => AccessMode::Read,
            NotifyAccessMode::Write => AccessMode::Write,
            NotifyAccessMode::Execute => AccessMode::Execute,
            NotifyAccessMode::Other => AccessMode::Other,
            NotifyAccessMode::Any => AccessMode::Unknown,
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
