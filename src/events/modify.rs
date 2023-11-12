use notify::event::{DataChange, MetadataKind};
use pyo3::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
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

#[derive(Debug)]
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

impl ModifyDataEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf, modify_kind: DataChange) -> Self {
        Self {
            detected_at_ns,
            path,
            data_type: DataType::from(modify_kind),
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyMetadataEvent {
    detected_at_ns: u128,
    path: PathBuf,
    metadata_type: MetadataType,
}

impl ModifyMetadataEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf, metadata_kind: MetadataKind) -> Self {
        Self {
            detected_at_ns,
            path,
            metadata_type: MetadataType::from(metadata_kind),
        }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyOtherEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

impl ModifyOtherEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}

#[pyclass]
#[derive(Debug)]
pub(crate) struct ModifyAnyEvent {
    detected_at_ns: u128,
    path: PathBuf,
}

impl ModifyAnyEvent {
    pub fn new(detected_at_ns: u128, path: PathBuf) -> Self {
        Self { detected_at_ns, path }
    }
}
