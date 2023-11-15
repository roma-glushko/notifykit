use notify::event::RenameMode;
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub enum RenameType {
    From = 0,
    To = 1,
    Both = 2,
    Other = 3,
    Any = 4,
}

impl From<RenameMode> for RenameType {
    fn from(mode: RenameMode) -> Self {
        return match mode {
            RenameMode::From => RenameType::From,
            RenameMode::To => RenameType::To,
            RenameMode::Both => RenameType::Both,
            RenameMode::Other => RenameType::Other,
            RenameMode::Any => RenameType::Any,
        };
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct RenameEvent {
    #[pyo3(get)]
    pub detected_at_ns: u128,
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub rename_type: RenameType,
    #[pyo3(get)]
    pub target_path: Option<PathBuf>,
}

#[pymethods]
impl RenameEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf, rename_type: RenameType, target_path: Option<PathBuf>) -> Self {
        Self {
            detected_at_ns,
            path,
            target_path,
            rename_type,
        }
    }
}

pub fn from_rename_mode(
    detected_at_ns: u128,
    path: PathBuf,
    rename_mode: RenameMode,
    target_path: Option<PathBuf>,
) -> RenameEvent {
    RenameEvent {
        detected_at_ns,
        path,
        rename_type: RenameType::from(rename_mode),
        target_path,
    }
}
