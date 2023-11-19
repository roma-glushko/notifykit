mod events;
mod watcher;
extern crate notify;
extern crate pyo3;

use crate::watcher::{Watcher, WatcherError};
use pyo3::exceptions::PyKeyboardInterrupt;
use pyo3::prelude::*;
use std::time::Duration;

use crate::events::access::AccessEvent;
use crate::events::create::CreateEvent;
use crate::events::delete::DeleteEvent;
use crate::events::modify::{ModifyAnyEvent, ModifyDataEvent, ModifyMetadataEvent, ModifyOtherEvent};
use crate::events::rename::RenameEvent;

#[pyclass]
pub struct WatcherWrapper {
    watcher: Watcher,
}

#[pymethods]
impl WatcherWrapper {
    #[new]
    fn __init__(debounce_ms: u64, debug: bool, debounce_tick_rate_ms: Option<u64>) -> PyResult<Self> {
        let watcher = Watcher::new(debounce_ms, debounce_tick_rate_ms, debug);

        Ok(WatcherWrapper { watcher: watcher? })
    }

    pub fn get(&self, py: Python) -> PyResult<Option<PyObject>> {
        loop {
            match py.check_signals() {
                Ok(_) => (),
                Err(_) => {
                    return Err(PyKeyboardInterrupt::new_err("KeyboardInterrupt"));
                }
            };

            let result = self.watcher.get(Duration::from_millis(200));

            match result {
                Ok(event_or_none) => {
                    return match event_or_none {
                        Some(event) => Ok(Some(event.to_object(py))),
                        None => Ok(None),
                    }
                }
                Err(_) => continue,
            }
        }
    }

    pub fn start(&mut self, py: Python) -> PyResult<()> {
        py.allow_threads(|| self.watcher.start(400));

        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        self.watcher.stop();

        Ok(())
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

    // Event Data Classes
    m.add_class::<AccessEvent>()?;
    m.add_class::<CreateEvent>()?;
    m.add_class::<DeleteEvent>()?;
    m.add_class::<RenameEvent>()?;
    m.add_class::<ModifyMetadataEvent>()?;
    m.add_class::<ModifyDataEvent>()?;
    m.add_class::<ModifyOtherEvent>()?;
    m.add_class::<ModifyAnyEvent>()?;

    Ok(())
}
