mod events;
mod file_cache;
mod processor;
mod watcher;

use crate::events::EventType;
use crate::watcher::{Watcher, WatcherError};
use pyo3::exceptions::{PyOSError, PyStopAsyncIteration};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::sync::{Arc, Mutex};
use tokio::runtime::Builder;
use tokio::sync::broadcast;

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
    inner: Arc<Mutex<Watcher>>,
}

#[pymethods]
impl WatcherWrapper {
    #[new]
    fn __init__(debounce_ms: u64, event_buffer_size: usize) -> PyResult<Self> {
        let inner =
            Watcher::new(debounce_ms, event_buffer_size).map_err(|e| PyOSError::new_err(e.to_string()))?;

        Ok(WatcherWrapper {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    pub fn watch<'py>(
        &self,
        py: Python<'py>,
        paths: Vec<String>,
        recursive: bool,
        ignore_permission_errors: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let watcher = Arc::clone(&self.inner);

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = tokio::task::spawn_blocking(move || {
                let mut guard = watcher.lock().map_err(|e| PyOSError::new_err(e.to_string()))?;

                guard.watch(&paths, recursive, ignore_permission_errors)
            })
            .await;

            match res {
                Ok(inner) => inner,
                Err(join_err) => Err(PyOSError::new_err(join_err.to_string())),
            }
        })
    }

    pub fn unwatch<'py>(&mut self, py: Python<'py>, paths: Vec<String>) -> PyResult<Bound<'py, PyAny>> {
        let watcher = self.inner.clone();

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let res = tokio::task::spawn_blocking(move || {
                let mut guard = watcher.lock().map_err(|e| PyOSError::new_err(e.to_string()))?;

                guard.unwatch(paths)
            })
            .await;

            match res {
                Ok(inner) => inner,
                Err(join_err) => Err(PyOSError::new_err(join_err.to_string())),
            }
        })
    }

    fn events(&self, tick_ms: u64) -> PyResult<EventBatchIter> {
        let rx = {
            let mut g = self.inner.lock().map_err(|e| PyOSError::new_err(e.to_string()))?;
            g.start_drain(std::time::Duration::from_millis(tick_ms));
            g.subscribe()
        };

        Ok(EventBatchIter::new(rx))
    }

    pub fn stop(&self) {
        if let Ok(mut g) = self.inner.lock() {
            g.stop();
        }
    }

    pub fn __repr__(&mut self) -> PyResult<String> {
        let mut watcher = self.inner.lock().map_err(|e| PyOSError::new_err(e.to_string()))?;

        Ok(watcher.repr())
    }
}

#[pyclass]
struct EventBatchIter {
    rx: Arc<tokio::sync::Mutex<broadcast::Receiver<Vec<EventType>>>>,
}

impl EventBatchIter {
    fn new(rx: broadcast::Receiver<Vec<EventType>>) -> Self {
        Self {
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }
}

#[pymethods]
impl EventBatchIter {
    fn __aiter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __anext__<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let rx = Arc::clone(&self.rx);

        let fut = pyo3_async_runtimes::tokio::future_into_py(py, async move {
            loop {
                let mut guard = rx.lock().await;
                match guard.recv().await {
                    Ok(batch) => {
                        return Python::attach(|py| {
                            let list = PyList::new(py, &batch)?;
                            Ok(list.into_any().unbind())
                        });
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("consumer too slow, {n} event batch(es) dropped");
                        continue;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        return Err(PyErr::new::<PyStopAsyncIteration, _>("event stream closed"));
                    }
                }
            }
        })?;

        Ok(Some(fut))
    }
}

#[pymodule]
fn _notifykit_lib(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();

    let mut builder = Builder::new_multi_thread();
    builder.enable_all();
    pyo3_async_runtimes::tokio::init(builder);

    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<WatcherWrapper>()?;
    m.add_class::<EventBatchIter>()?;

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

    m.add_class::<RenameEvent>()?;

    Ok(())
}
