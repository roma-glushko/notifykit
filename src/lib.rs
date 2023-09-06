extern crate notify;
extern crate pyo3;

// use pyo3::create_exception;
// use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyPermissionError, PyRuntimeError, PyTypeError};
use pyo3::prelude::*;

// use notify::event::{Event, EventKind, ModifyKind, RenameMode};
// use notify::{
//     Config as NotifyConfig, ErrorKind as NotifyErrorKind, PollWatcher, RecommendedWatcher, RecursiveMode,
//     Result as NotifyResult, Watcher,
// };

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn _inotify_toolkit_lib(py: Python, m: &PyModule) -> PyResult<()> {
    let mut version = env!("CARGO_PKG_VERSION").to_string();
    version = version.replace("-alpha", "a").replace("-beta", "b");

    m.add("__version__", version)?;

    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}
