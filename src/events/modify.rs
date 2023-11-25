use notify::event::{DataChange, MetadataKind};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub enum MetadataType {
    UNKNOWN = 0,
    ACCESS_TIME = 1,
    WRITE_TIME = 2,
    OWNERSHIP = 3,
    PERMISSIONS = 4,
    EXTENDED = 5,
    OTHER = 6,
}

impl From<MetadataKind> for MetadataType {
    fn from(kind: MetadataKind) -> Self {
        match kind {
            MetadataKind::AccessTime => Self::ACCESS_TIME,
            MetadataKind::WriteTime => Self::WRITE_TIME,
            MetadataKind::Ownership => Self::OWNERSHIP,
            MetadataKind::Permissions => Self::PERMISSIONS,
            MetadataKind::Extended => Self::EXTENDED,
            MetadataKind::Other => Self::OWNERSHIP,
            MetadataKind::Any => Self::UNKNOWN,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum DataType {
    UNKNOWN = 0,
    CONTENT = 1,
    SIZE = 2,
    OTHER = 3,
}

impl From<DataChange> for DataType {
    fn from(kind: DataChange) -> Self {
        match kind {
            DataChange::Content => Self::CONTENT,
            DataChange::Size => Self::SIZE,
            DataChange::Other => Self::OTHER,
            DataChange::Any => Self::UNKNOWN,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ModifyDataEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub data_type: DataType,
}

#[pymethods]
impl ModifyDataEvent {
    #[new]
    pub fn new(path: PathBuf, data_type: DataType) -> Self {
        Self { path, data_type }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!(
            "ModifyDataEvent({:?}, {:?})",
            slf.borrow().path,
            slf.borrow().data_type,
        ))
    }
}

pub fn from_data_kind(path: PathBuf, data_kind: DataChange) -> ModifyDataEvent {
    ModifyDataEvent {
        path,
        data_type: DataType::from(data_kind),
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ModifyMetadataEvent {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub metadata_type: MetadataType,
}

#[pymethods]
impl ModifyMetadataEvent {
    #[new]
    pub fn new(path: PathBuf, metadata_type: MetadataType) -> Self {
        Self { path, metadata_type }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!(
            "ModifyMetadataEvent({:?}, {:?})",
            slf.borrow().path,
            slf.borrow().metadata_type,
        ))
    }
}

pub fn from_metadata_kind(path: PathBuf, metadata_kind: MetadataKind) -> ModifyMetadataEvent {
    ModifyMetadataEvent {
        path,
        metadata_type: MetadataType::from(metadata_kind),
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ModifyOtherEvent {
    #[pyo3(get)]
    pub path: PathBuf,
}

#[pymethods]
impl ModifyOtherEvent {
    #[new]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!("ModifyOtherEvent({:?})", slf.borrow().path,))
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct ModifyUnknownEvent {
    #[pyo3(get)]
    pub path: PathBuf,
}

#[pymethods]
impl ModifyUnknownEvent {
    #[new]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!("ModifyAnyEvent({:?})", slf.borrow().path,))
    }
}
