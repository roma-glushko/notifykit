use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Debug)]
pub(crate) struct EventAttributes {
    pub(crate) tracker: Option<usize>,
    // TODO: add the rest of data
}

#[pymethods]
impl EventAttributes {
    // pub(crate) fn from_raw_attrs(attrs: EventAttributes) -> Self {
    //     EventAttributes {
    //         tracker: attrs.tracker(),
    //     }
    // }
}
