use notify::event::{CreateKind, RemoveKind};
use pyo3::prelude::*;
use std::convert::From;

#[pyclass(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Clone)]
pub enum ObjectType {
    Unknown = 0,
    File = 1,
    Dir = 2,
    Other = 3,
}

impl From<CreateKind> for ObjectType {
    fn from(kind: CreateKind) -> Self {
        match kind {
            CreateKind::Any => ObjectType::Unknown,
            CreateKind::File => ObjectType::File,
            CreateKind::Folder => ObjectType::Dir,
            CreateKind::Other => ObjectType::Other,
        }
    }
}

impl From<RemoveKind> for ObjectType {
    fn from(kind: RemoveKind) -> Self {
        match kind {
            RemoveKind::Any => ObjectType::Unknown,
            RemoveKind::File => ObjectType::File,
            RemoveKind::Folder => ObjectType::Dir,
            RemoveKind::Other => ObjectType::Other,
        }
    }
}
