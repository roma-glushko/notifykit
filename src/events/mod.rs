use pyo3::conversion::IntoPyObject;
use pyo3::prelude::*;

pub(crate) mod access;
pub(crate) mod base;
pub(crate) mod create;
pub(crate) mod delete;
pub(crate) mod modify;
pub(crate) mod rename;

#[derive(Debug, Clone)]
pub enum EventType {
    Access(access::AccessEvent),
    Create(create::CreateEvent),
    Delete(delete::DeleteEvent),
    ModifyMetadata(modify::ModifyMetadataEvent),
    ModifyData(modify::ModifyDataEvent),
    ModifyUnknown(modify::ModifyUnknownEvent),
    ModifyOther(modify::ModifyOtherEvent),
    Rename(rename::RenameEvent),
}

impl<'py> IntoPyObject<'py> for &EventType {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Bound<'py, PyAny>, PyErr> {
        Ok(match self {
            EventType::Access(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::Create(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::Delete(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::ModifyMetadata(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::ModifyData(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::ModifyOther(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::ModifyUnknown(event) => Bound::new(py, event.clone())?.into_any(),
            EventType::Rename(event) => Bound::new(py, event.clone())?.into_any(),
        })
    }
}
