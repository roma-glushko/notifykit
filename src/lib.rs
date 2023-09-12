mod events;
mod watcher;

extern crate notify;
extern crate pyo3;

use crate::events::RawEvent;
use crate::watcher::{Watcher, WatcherError};
use pyo3::prelude::*;

#[pymodule]
fn _inotify_toolkit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add("WatcherError", py.get_type::<WatcherError>())?;

    m.add_class::<Watcher>()?;

    // Event Data Classes

    m.add_class::<RawEvent>()?;

    Ok(())
}
