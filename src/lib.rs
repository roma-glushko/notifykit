mod watcher;
mod events;

extern crate notify;
extern crate pyo3;

use pyo3::prelude::*;
use crate::watcher::{Watcher, WatcherError};
use crate::events::{Event, CreateEvent, RemoveEvent, ModifyEvent, OtherEvent, FileCreatedEvent, DirCreatedEvent, OtherCreatedEvent, FileRemovedEvent, DirRemovedEvent, OtherRemovedEvent};

#[pymodule]
fn _inotify_toolkit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<Watcher>()?;

    // Event Data Classes

    m.add_class::<Event>()?;
    m.add_class::<CreateEvent>()?;
    m.add_class::<RemoveEvent>()?;
    m.add_class::<ModifyEvent>()?;
    m.add_class::<OtherEvent>()?;

    m.add_class::<FileCreatedEvent>()?;
    m.add_class::<DirCreatedEvent>()?;
    m.add_class::<OtherCreatedEvent>()?;

    m.add_class::<FileRemovedEvent>()?;
    m.add_class::<DirRemovedEvent>()?;
    m.add_class::<OtherRemovedEvent>()?;

    Ok(())
}
