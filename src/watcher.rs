extern crate notify;
extern crate pyo3;

use std::path::Path;
use std::time::Duration;
// use pyo3::exceptions::{PyFileNotFoundError, PyOSError, PyPermissionError, PyRuntimeError, PyTypeError};
use pyo3::exceptions::{PyFileNotFoundError, PyException, PyPermissionError, PyOSError};
use pyo3::prelude::*;

use notify::event::{Event, EventKind, ModifyKind, RenameMode};
use notify::{
    Config as NotifyConfig, ErrorKind as NotifyErrorKind, PollWatcher, RecommendedWatcher, RecursiveMode,
    Result as NotifyResult, Watcher as NotifyWatcher,
};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

#[derive(Debug)]
enum WatcherType {
    None,
    Poll(PollWatcher),
    Recommended(RecommendedWatcher),
}

#[pyclass]
pub(crate) struct Watcher {
    debug: bool,
    watcher: WatcherType,
}

#[pymethods]
impl Watcher {
    #[new]
    fn py_new(
        debug: bool,
        force_polling: bool,
        poll_delay_ms: u64,
    ) -> PyResult<Self> {
       if force_polling {
           return Watcher::create_poll_watcher(debug, poll_delay_ms)
       }

        return Watcher::create_recommended_watcher(debug, poll_delay_ms)
    }

    fn create_poll_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Self> {
        if watch_paths.iter().any(|p| !Path::new(p).exists()) {
            return Err(PyFileNotFoundError::new_err("No such file or directory"));
        }

        let delay = Duration::from_millis(poll_delay_ms);
        let config = NotifyConfig::default().with_poll_interval(delay);

        let watcher = match PollWatcher::new(event_handler, config) {
            Ok(watcher) => watcher,
            Err(e) => return Err(WatcherError::new_err(format!("Error creating poll watcher: {}", e)))
        };

        Ok(Watcher{
            debug,
            watcher: WatcherType::Poll(watcher),
        })
    }

    fn create_recommended_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Self> {
        let watcher = match RecommendedWatcher::new(
            Watcher::handle_event,
            NotifyConfig::default(),
        ) {
            Ok(watcher) => watcher,
            Err(error) => {
                return match &error.kind {
                    NotifyErrorKind::Io(notify_error) => {
                        if notify_error.raw_os_error() == Some(38) {
                            // fall back to PollWatcher

                            if debug {
                                eprintln!(
                                    "Error using recommend watcher: {:?}, falling back to PollWatcher",
                                    notify_error
                                );
                            }

                            return Watcher::create_poll_watcher(debug, poll_delay_ms)
                        }

                        Err(WatcherError::new_err(format!("Error creating recommended watcher: {}", error)))
                    }
                    _ => {
                        Err(WatcherError::new_err(format!("Error creating recommended watcher: {}", error)))
                    }
                }
            }
        };

        Ok(Watcher{
            debug,
            watcher: WatcherType::Recommended(watcher)
        })
    }

    fn handle_event(res: NotifyResult<Event>) {
        match res {
            Ok(event) => {
                if let Some(path_buf) = event.paths.first() {
                    let path = match path_buf.to_str() {
                        Some(s) => s.to_string(),
                        None => {
                            return;
                        }
                    };

                    let change = match event.kind {

                    };
                }
            }
            Err(e) => {
                // TODO: do something about it
            }
        }
    }

    fn map_notify_error(notify_error: notify::Error) -> PyErr {
        let err_string = notify_error.to_string();

        match notify_error.kind {
            NotifyErrorKind::PathNotFound => return PyFileNotFoundError::new_err(err_string),
            NotifyErrorKind::Generic(ref err) => {
                // on Windows, we get a Generic with this message when the path does not exist
                if err.as_str() == "Input watch path is neither a file nor a directory." {
                    return PyFileNotFoundError::new_err(err_string);
                }
            }
            NotifyErrorKind::Io(ref io_error) => match io_error.kind() {
                IOErrorKind::NotFound => return PyFileNotFoundError::new_err(err_string),
                IOErrorKind::PermissionDenied => return PyPermissionError::new_err(err_string),
                _ => (),
            },
            _ => (),
        };

        PyOSError::new_err(format!("{} ({:?})", err_string, error))
    }

    pub fn watch(slf: &PyCell<Self>, paths: Vec<String>, recursive: bool, ignore_permission_errors: bool) -> PyResult<Self> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        let mut watcher = match slf.borrow().watcher {
            WatcherType::None => return Err(WatcherError::new_err("Watcher is closed")),
            WatcherType::Poll(w) => w,
            WatcherType::Recommended(w) => w,
        };

        for path in paths.into_iter() {
            let result = watcher.watch(Path::new(&path), mode);

            match result {
                Err(err) => {
                    if !ignore_permission_errors {
                        return Err( Watcher::map_notify_error(err));
                    }
                }
                _ => (),
            }
        }

        if slf.borrow().debug {
            eprintln!("watcher: {:?}", watcher);
        }

        Ok(slf)
    }

    // pub fn unwatch(slf: &PyCell<Self>) -> PyResult<Self> {
    //     // TODO: implement
    //     slf
    // }
    //
    // pub fn get(slf: &PyCell<Self>) -> PyResult<Self> {
    //     // TODO: implement
    // }

    /// https://github.com/PyO3/pyo3/issues/1205#issuecomment-1164096251 for advice on `__enter__`
    pub fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    pub fn close(&mut self) {
        self.watcher = WatcherEnum::None;
    }

    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        self.close();
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Watcher({:#?})", self.watcher))
    }
}
