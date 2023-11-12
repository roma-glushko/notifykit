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
#[derive(Debug)]
pub struct RenameEvent {
    detected_at_ns: u128,
    path: PathBuf,
    target_path: Option<PathBuf>,
    rename_type: RenameType,
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
    target_path: Option<PathBuf>,
    rename_mode: RenameMode,
) -> RenameEvent {
    RenameEvent {
        detected_at_ns,
        path,
        rename_type: RenameType::from(rename_mode),
        target_path,
    }
}
