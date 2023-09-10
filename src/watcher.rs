extern crate notify;
extern crate pyo3;

use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use std::io::ErrorKind as IOErrorKind;
use pyo3::exceptions::{PyFileNotFoundError, PyException, PyPermissionError, PyOSError};
use pyo3::prelude::*;

// use notify::event::{Event, EventKind, ModifyKind, RenameMode};
use notify::event::{Event};
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
    receiver: Receiver<NotifyResult<Event>>,
    watcher: WatcherType,
}

fn create_poll_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
    let (sender, receiver) = std::sync::mpsc::channel();
    let delay = Duration::from_millis(poll_delay_ms);
    let config = NotifyConfig::default().with_poll_interval(delay);

    let watcher = match PollWatcher::new(sender, config) {
        Ok(watcher) => watcher,
        Err(e) => return Err(WatcherError::new_err(format!("Error creating poll watcher: {}", e)))
    };

    Ok(Watcher{
        debug,
        receiver,
        watcher: WatcherType::Poll(watcher),
    })
}

fn create_recommended_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
    let (sender, receiver) = std::sync::mpsc::channel();

    let watcher = match RecommendedWatcher::new(
        sender,
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

                        return create_poll_watcher(debug, poll_delay_ms)
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
        receiver,
        watcher: WatcherType::Recommended(watcher)
    })
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

    PyOSError::new_err(format!("{} ({:?})", err_string, notify_error))
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
           return create_poll_watcher(debug, poll_delay_ms)
       }

        return create_recommended_watcher(debug, poll_delay_ms)
    }

    pub fn watch(&mut self, paths: Vec<String>, recursive: bool, ignore_permission_errors: bool) -> PyResult<()> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for path_str in paths.into_iter() {
            let path = Path::new(&path_str);

            if !path.exists() {
                return Err(PyFileNotFoundError::new_err("No such file or directory"));
            }

            let result = match self.watcher {
                WatcherType::Recommended(ref mut w) => w.watch(path, mode),
                WatcherType::Poll(ref mut w) => w.watch(path, mode),
                WatcherType::None => return Err(WatcherError::new_err("Watcher is closed")),
            };

            match result {
                Err(err) => {
                    if !ignore_permission_errors {
                        return Err(map_notify_error(err));
                    }
                }
                _ => (),
            }
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.watcher);
        }

        Ok(())
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        for path_str in paths.into_iter() {
            let path = Path::new(&path_str);

            let result = match self.watcher {
                WatcherType::Recommended(ref mut w) => w.unwatch(path),
                WatcherType::Poll(ref mut w) => w.unwatch(path),
                WatcherType::None => return Err(WatcherError::new_err("Watcher is closed")),
            };

            match result {
                Err(err) => {
                    return Err(map_notify_error(err));
                }
                _ => (),
            }
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.watcher);
        }

        Ok(())
    }

    // fn handle_event(res: NotifyResult<Event>) {
    //     match res {
    //         Ok(event) => {
    //             if let Some(path_buf) = event.paths.first() {
    //                 let path = match path_buf.to_str() {
    //                     Some(s) => s.to_string(),
    //                     None => {
    //                         return;
    //                     }
    //                 };
    //
    //                 let change = match event.kind {
    //
    //                 };
    //             }
    //         }
    //         Err(e) => {
    //             // TODO: do something about it
    //         }
    //     }
    // }

    pub fn get(&self) -> PyResult<()> {
        for event in &self.receiver {
            println!("{:?}", event);
        }

        Ok(())
    }

    pub fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    pub fn close(&mut self) {
        self.watcher = WatcherType::None;
    }

    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_value: PyObject, _traceback: PyObject) {
        self.close();
    }

    pub fn __repr__(&self) -> PyResult<String> {
        Ok(format!("Watcher({:#?})", self.watcher))
    }
}
