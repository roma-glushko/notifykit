mod events;
mod watcher;

extern crate notify;
extern crate pyo3;

use crate::events::RawEvent;
use crate::watcher::{Watcher, WatcherError};
use pyo3::prelude::*;

#[pyclass]
pub(crate) struct WatcherWrapper {
    watcher: Watcher,
}

#[pymethods]
impl WatcherWrapper {
    #[new]
    fn __init__(debug: bool, force_polling: bool, poll_delay_ms: u64) -> PyResult<Self> {
        let watcher = Watcher::new(debug, force_polling, poll_delay_ms);

        return Ok(WatcherWrapper { watcher: watcher? });
    }

    pub fn __enter__(&mut self, slf: Py<Self>) -> Py<Self> {
        self.watcher.start();

        slf
    }

    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        self.watcher.stop()
    }

    pub fn watch(&mut self, paths: Vec<String>, recursive: bool, ignore_permission_errors: bool) -> PyResult<()> {
        Ok(self.watcher.watch(paths, recursive, ignore_permission_errors)?)
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        Ok(self.watcher.unwatch(paths)?)
    }

    pub fn __repr__(&mut self) -> PyResult<String> {
        Ok(self.watcher.repr())
    }
}

#[pymodule]
fn _inotify_toolkit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<WatcherWrapper>()?;

    // Event Data Classes

    m.add_class::<RawEvent>()?;

    Ok(())
}
