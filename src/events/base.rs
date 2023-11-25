use notify::event::{CreateKind, RemoveKind};
use pyo3::prelude::*;
use std::convert::From;

#[pyclass]
#[derive(Debug, Clone)]
pub enum ObjectType {
    UNKNOWN = 0,
    FILE = 1,
    DIR = 2,
    OTHER = 3,
}

impl From<CreateKind> for ObjectType {
    fn from(kind: CreateKind) -> Self {
        match kind {
            CreateKind::Any => ObjectType::UNKNOWN,
            CreateKind::File => ObjectType::FILE,
            CreateKind::Folder => ObjectType::DIR,
            CreateKind::Other => ObjectType::OTHER,
        }
    }
}

impl From<RemoveKind> for ObjectType {
    fn from(kind: RemoveKind) -> Self {
        match kind {
            RemoveKind::Any => ObjectType::UNKNOWN,
            RemoveKind::File => ObjectType::FILE,
            RemoveKind::Folder => ObjectType::DIR,
            RemoveKind::Other => ObjectType::OTHER,
        }
    }
}
