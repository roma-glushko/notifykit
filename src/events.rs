use pyo3::prelude::*;
use notify::event::{EventAttributes as NotifyEventAttrs};

enum EventTypes {
    Other  = 0b1000000000000000,
    Create = 0b0100000000000000,
    Access = 0b0010000000000000,
    Remove = 0b0001000000000000,
    Modify = 0b0000100000000000,
}

enum EventTypeAttributes {
    File      = 0b0000010000000000,
    Dir       = 0b0000001000000000,
    Other     = 0b0000000100000000,
    Data      = 0b0000000010000000,
    Metadata  = 0b0000000001000000,
    Name      = 0b0000000000100000,
}

#[pyclass]
#[derive(Clone)]
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

#[pyclass(subclass)]
pub(crate) struct Event {
    event_type: u64,
    detected_at_ns: u128,
    path: String,
    attributes: EventAttributes,
}

#[pymethods]
impl Event {
    #[new]
    fn new(event_type: u64, detected_at_ns: u128, path: String, attributes: EventAttributes) -> Self {
        Event {
            event_type,
            detected_at_ns,
            path,
            attributes,
        }
    }
}

#[pyclass(extends=Event, subclass)]
struct AccessEvent {

}

#[pymethods]
impl AccessEvent {
    #[new]
    fn new(event_type: u64, detected_at_ns: u128, path: String, attributes: EventAttributes) -> (Self, Event) {
        (
            AccessEvent { },
            Event::new(event_type | EventTypes::Access as u64, detected_at_ns, path, attributes)
        )
    }
}

#[pyclass(extends=Event, subclass)]
pub(crate) struct CreateEvent {

}

#[pymethods]
impl CreateEvent {
    #[new]
    pub(crate) fn new(event_type: u64, detected_at_ns: u128, path: String, attributes: EventAttributes) -> (Self, Event) {
        (
            CreateEvent { },
            Event::new(event_type | EventTypes::Create as u64, detected_at_ns, path, attributes)
        )
    }
}

#[pyclass(extends=Event, subclass)]
pub(crate) struct RemoveEvent {

}

#[pymethods]
impl RemoveEvent {
    #[new]
    pub(crate) fn new(event_type: u64, detected_at_ns: u128, path: String, attributes: EventAttributes) -> (Self, Event) {
        (
            RemoveEvent { },
            Event::new(event_type | EventTypes::Remove as u64, detected_at_ns, path, attributes)
        )
    }
}

#[pyclass(extends=Event, subclass)]
pub(crate) struct ModifyEvent {
}

#[pymethods]
impl ModifyEvent {
    #[new]
    fn new(event_type: u64, detected_at_ns: u128, path: String, attributes: EventAttributes) -> (Self, Event) {
        (
            ModifyEvent { },
            Event::new(event_type | EventTypes::Modify as u64, detected_at_ns, path, attributes)
        )
    }
}

#[pyclass(extends=Event, subclass)]
pub(crate) struct OtherEvent {

}

#[pymethods]
impl OtherEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, attributes: EventAttributes) -> (Self, Event) {
        (
            OtherEvent { },
            Event::new(0 | EventTypes::Other as u64, detected_at_ns, path, attributes)
        )
    }
}

#[pyclass(extends=CreateEvent)]
pub(crate) struct FileCreatedEvent {
}

#[pymethods]
impl FileCreatedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, attributes: EventAttributes) -> PyClassInitializer<Self> {
        let create_event = CreateEvent::new(
            0 | EventTypeAttributes::File as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(create_event).add_subclass(FileCreatedEvent {})
    }
}

#[pyclass(extends=CreateEvent)]
pub(crate) struct DirCreatedEvent {
}

#[pymethods]
impl DirCreatedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, attributes: EventAttributes) -> PyClassInitializer<Self> {
        let create_event = CreateEvent::new(
            0 | EventTypeAttributes::Dir as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(create_event).add_subclass(DirCreatedEvent {})
    }
}

#[pyclass(extends=CreateEvent)]
pub(crate) struct OtherCreatedEvent {
}

#[pymethods]
impl OtherCreatedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, attributes: EventAttributes) -> PyClassInitializer<Self> {
        let create_event = CreateEvent::new(
            0 | EventTypeAttributes::Other as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(create_event).add_subclass(OtherCreatedEvent {})
    }
}

#[pyclass(extends=RemoveEvent)]
pub(crate) struct FileRemovedEvent {
}

#[pymethods]
impl FileRemovedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, attributes: EventAttributes) -> PyClassInitializer<Self> {
        let remove_event = RemoveEvent::new(
            0 | EventTypeAttributes::File as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(remove_event).add_subclass(FileRemovedEvent {})
    }
}

#[pyclass(extends=RemoveEvent)]
pub(crate) struct DirRemovedEvent {
}

#[pymethods]
impl DirRemovedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, mut attributes: EventAttributes) -> PyClassInitializer<Self> {
        let remove_event = RemoveEvent::new(
            0 | EventTypeAttributes::Dir as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(remove_event).add_subclass(DirRemovedEvent {})
    }
}

#[pyclass(extends=RemoveEvent)]
pub(crate) struct OtherRemovedEvent {
}

#[pymethods]
impl OtherRemovedEvent {
    #[new]
    pub(crate) fn new(detected_at_ns: u128, path: String, mut attributes: EventAttributes) -> PyClassInitializer<Self> {
        let remove_event = RemoveEvent::new(
            0 | EventTypeAttributes::Other as u64,
            detected_at_ns,
            path,
            attributes,
        );

        PyClassInitializer::from(remove_event).add_subclass(OtherRemovedEvent {})
    }
}


