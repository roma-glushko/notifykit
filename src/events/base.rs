use notify::event::{CreateKind, RemoveKind};
use std::convert::From;

#[derive(Debug)]
pub(crate) enum ObjectType {
    Any = 0,
    File = 1,
    Dir = 2,
    Other = 3,
}

impl From<CreateKind> for ObjectType {
    fn from(kind: CreateKind) -> Self {
        return match kind {
            CreateKind::Any => ObjectType::Any,
            CreateKind::File => ObjectType::File,
            CreateKind::Folder => ObjectType::Dir,
            CreateKind::Other => ObjectType::Other,
        };
    }
}

impl From<RemoveKind> for ObjectType {
    fn from(kind: RemoveKind) -> Self {
        return match kind {
            RemoveKind::Any => ObjectType::Any,
            RemoveKind::File => ObjectType::File,
            RemoveKind::Folder => ObjectType::Dir,
            RemoveKind::Other => ObjectType::Other,
        };
    }
}
