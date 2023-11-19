use notify::event::{DataChange, MetadataKind};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub enum MetadataType {
    AccessTime = 0,
    WriteTime = 1,
    Ownership = 2,
    Permissions = 3,
    Extended = 4,
    Other = 5,
    Any = 6,
}

impl From<MetadataKind> for MetadataType {
    fn from(kind: MetadataKind) -> Self {
        match kind {
            MetadataKind::AccessTime => Self::AccessTime,
            MetadataKind::WriteTime => Self::WriteTime,
            MetadataKind::Ownership => Self::Ownership,
            MetadataKind::Permissions => Self::Permissions,
            MetadataKind::Extended => Self::Extended,
            MetadataKind::Other => Self::Other,
            MetadataKind::Any => Self::Any,
        }
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum DataType {
    Any = 0,
    Content = 1,
    Size = 2,
    Other = 3,
}

impl From<DataChange> for DataType {
    fn from(kind: DataChange) -> Self {
        match kind {
            DataChange::Content => Self::Content,
            DataChange::Size => Self::Size,
            DataChange::Other => Self::Other,
            DataChange::Any => Self::Any,
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
pub struct ModifyAnyEvent {
    #[pyo3(get)]
    pub path: PathBuf,
}

#[pymethods]
impl ModifyAnyEvent {
    #[new]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        Ok(format!("ModifyAnyEvent({:?})", slf.borrow().path,))
    }
}
