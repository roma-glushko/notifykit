use crate::events::base::Event;
use notify::event::{DataChange, MetadataKind};
use pyo3::prelude::*;
use std::path::PathBuf;

#[pyclass]
#[derive(Debug, Clone)]
pub(crate) enum MetadataType {
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
        return match kind {
            MetadataKind::AccessTime => Self::AccessTime,
            MetadataKind::WriteTime => Self::WriteTime,
            MetadataKind::Ownership => Self::Ownership,
            MetadataKind::Permissions => Self::Permissions,
            MetadataKind::Extended => Self::Extended,
            MetadataKind::Other => Self::Other,
            MetadataKind::Any => Self::Any,
        };
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub(crate) enum DataType {
    Any = 0,
    Content = 1,
    Size = 2,
    Other = 3,
}

impl From<DataChange> for DataType {
    fn from(kind: DataChange) -> Self {
        return match kind {
            DataChange::Content => Self::Content,
            DataChange::Size => Self::Size,
            DataChange::Other => Self::Other,
            DataChange::Any => Self::Any,
        };
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyDataEvent {
    detected_at_ns: u128,
    path: PathBuf,
    data_type: DataType,
}

#[pymethods]
impl ModifyDataEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf, data_type: DataType) -> Self {
        Self {
            detected_at_ns,
            path,
            data_type,
        }
    }
}

impl Event for ModifyDataEvent {}

pub fn from_data_kind(detected_at_ns: u128, path: PathBuf, data_kind: DataChange) -> ModifyDataEvent {
    ModifyDataEvent {
        detected_at_ns,
        path,
        data_type: DataType::from(data_kind),
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyMetadataEvent {
    detected_at_ns: u128,
    path: PathBuf,
    metadata_type: MetadataType,
}

#[pymethods]
impl ModifyMetadataEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf, metadata_type: MetadataType) -> Self {
        Self {
            detected_at_ns,
            path,
            metadata_type,
        }
    }
}

impl Event for ModifyMetadataEvent {}

pub fn from_metadata_kind(detected_at_ns: u128, path: PathBuf, metadata_kind: MetadataKind) -> ModifyMetadataEvent {
    ModifyMetadataEvent {
        detected_at_ns,
        path,
        metadata_type: MetadataType::from(metadata_kind),
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyOtherEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

#[pymethods]
impl ModifyOtherEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}

impl Event for ModifyOtherEvent {}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyAnyEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

#[pymethods]
impl ModifyAnyEvent {
    #[new]
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}

impl Event for ModifyAnyEvent {}
