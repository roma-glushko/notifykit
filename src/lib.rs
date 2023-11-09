mod events;
mod events2;
mod watcher;

extern crate notify;
extern crate pyo3;

use crate::events::RawEvent;
use crate::watcher::{Watcher, WatcherError};
use pyo3::exceptions::PyKeyboardInterrupt;
use pyo3::prelude::*;
use std::time::Duration;

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

    pub fn get(&self, py: Python) -> PyResult<Option<RawEvent>> {
        loop {
            match py.check_signals() {
                Ok(_) => (),
                Err(_) => {
                    // self.clear();
                    return Err(PyKeyboardInterrupt::new_err("KeyboardInterrupt"));
                }
            };

            let result = self.watcher.get(Duration::from_millis(200));

            match result {
                Ok(e) => return Ok(e),
                Err(_) => continue,
            }
        }
    }

    pub fn start(&mut self, py: Python) -> PyResult<()> {
        py.allow_threads(|| self.watcher.start());

        Ok(())
    }

    pub fn stop(&mut self) -> PyResult<()> {
        self.watcher.stop();

        Ok(())
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
fn _inotifykit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<WatcherWrapper>()?;

    // Event Data Classes

    m.add_class::<RawEvent>()?;

    Ok(())
}
