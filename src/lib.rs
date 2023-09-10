mod watcher;

extern crate notify;
extern crate pyo3;

use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyPermissionError, PyRuntimeError, PyTypeError};


create_exception!(
    _inotify_toolkit_lib,
    WatchfilesRustInternalError,
    PyRuntimeError,
    "Internal or filesystem error."
);

#[pymodule]
fn _inotify_toolkit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add_class::<watcher::Watcher>()?;

    Ok(())
}
