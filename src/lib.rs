mod events;
mod file_cache;
mod processor;
mod watcher;

extern crate notify;
extern crate pyo3;

use crate::watcher::{Watcher, WatcherError};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use std::time::Duration;

use crate::events::access::{AccessEvent, AccessMode, AccessType};
use crate::events::base::ObjectType;
use crate::events::create::CreateEvent;
use crate::events::delete::DeleteEvent;
use crate::events::modify::{
    DataType, MetadataType, ModifyDataEvent, ModifyMetadataEvent, ModifyOtherEvent, ModifyUnknownEvent,
};
use crate::events::rename::RenameEvent;

#[pyclass]
pub struct WatcherWrapper {
    watcher: Watcher,
}

#[pymethods]
impl WatcherWrapper {
    #[new]
    fn __init__(debounce_ms: u64, debug: bool) -> PyResult<Self> {
        let watcher = Watcher::new(debounce_ms, debug);

        Ok(WatcherWrapper { watcher: watcher? })
    }

    pub fn get(&self, py: Python, tick_ms: u64, stop_event: PyObject) -> PyResult<Option<Vec<PyObject>>> {
        let is_stopping: Option<&PyAny> = match stop_event.is_none(py) {
            true => None,
            false => {
                let event: &PyAny = stop_event.extract(py)?;
                let func: &PyAny = event.getattr("is_set")?.extract()?;
                if !func.is_callable() {
                    return Err(PyTypeError::new_err("'stop_event.is_set' must be callable"));
                }
                Some(func)
            }
        };

        loop {
            py.allow_threads(|| std::thread::sleep(Duration::from_millis(tick_ms)));
            py.check_signals()?;

            if let Some(is_set) = is_stopping {
                if is_set.call0()?.is_true()? {
                    return Ok(None);
                }
            }

            let events = self.watcher.get();

            if events.is_empty() {
                continue;
            }

            let mut py_events = Vec::with_capacity(events.len());

            for event in events.iter() {
                py_events.push(event.to_object(py))
            }

            return Ok(Some(py_events));
        }
    }

    pub fn watch(&mut self, paths: Vec<String>, recursive: bool, ignore_permission_errors: bool) -> PyResult<()> {
        self.watcher.watch(paths, recursive, ignore_permission_errors)
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        self.watcher.unwatch(paths)
    }

    pub fn __repr__(&mut self) -> PyResult<String> {
        Ok(self.watcher.repr())
    }
}

#[pymodule]
fn _notifykit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<WatcherWrapper>()?;

    // Create & Delete Events
    m.add_class::<ObjectType>()?;
    m.add_class::<CreateEvent>()?;
    m.add_class::<DeleteEvent>()?;

    // Access Event
    m.add_class::<AccessType>()?;
    m.add_class::<AccessMode>()?;
    m.add_class::<AccessEvent>()?;

    // Modify Event
    m.add_class::<MetadataType>()?;
    m.add_class::<DataType>()?;

    m.add_class::<ModifyMetadataEvent>()?;
    m.add_class::<ModifyDataEvent>()?;
    m.add_class::<ModifyOtherEvent>()?;
    m.add_class::<ModifyUnknownEvent>()?;

    // Raname
    m.add_class::<RenameEvent>()?;

    Ok(())
}
