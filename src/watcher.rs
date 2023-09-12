extern crate notify;
extern crate pyo3;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use std::collections::VecDeque;
use std::io::ErrorKind as IOErrorKind;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crate::events::{
    new_access_event, new_create_event, new_modify_data_event, new_modify_event,
    new_modify_metadata_event, new_other_event, new_remove_event, new_rename_event,
    new_unknown_event, AccessMode, AccessType, DataChangeType, EventAttributes, EventType,
    MetadataType, ModifyType, ObjectType, RawEvent, RenameType,
};
use notify::event::{
    AccessKind, CreateKind, DataChange, Event as NotifyEvent, MetadataKind, ModifyKind, RemoveKind,
    RenameMode,
};
use notify::{
    Config as NotifyConfig, ErrorKind as NotifyErrorKind, EventKind, PollWatcher,
    RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher as NotifyWatcher,
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
    event_receiver: Receiver<NotifyResult<NotifyEvent>>,
    events: Arc<Mutex<VecDeque<RawEvent>>>,
    watcher: WatcherType,
}

fn get_current_time_ns() -> u128 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

fn create_poll_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
    let (event_sender, event_receiver) = std::sync::mpsc::channel();
    let delay = Duration::from_millis(poll_delay_ms);
    let config = NotifyConfig::default().with_poll_interval(delay);

    let watcher = match PollWatcher::new(event_sender, config) {
        Ok(watcher) => watcher,
        Err(e) => {
            return Err(WatcherError::new_err(format!(
                "Error creating poll watcher: {}",
                e
            )))
        }
    };

    Ok(Watcher {
        debug,
        event_receiver,
        events: Arc::new(Mutex::new(VecDeque::new())),
        watcher: WatcherType::Poll(watcher),
    })
}

fn create_recommended_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
    let (event_sender, event_receiver) = std::sync::mpsc::channel();

    let watcher = match RecommendedWatcher::new(event_sender, NotifyConfig::default()) {
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

                        return create_poll_watcher(debug, poll_delay_ms);
                    }

                    Err(WatcherError::new_err(format!(
                        "Error creating recommended watcher: {}",
                        error
                    )))
                }
                _ => Err(WatcherError::new_err(format!(
                    "Error creating recommended watcher: {}",
                    error
                ))),
            };
        }
    };

    Ok(Watcher {
        debug,
        event_receiver,
        events: Arc::new(Mutex::new(VecDeque::new())),
        watcher: WatcherType::Recommended(watcher),
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
    fn py_new(debug: bool, force_polling: bool, poll_delay_ms: u64) -> PyResult<Self> {
        if force_polling {
            return create_poll_watcher(debug, poll_delay_ms);
        }

        return create_recommended_watcher(debug, poll_delay_ms);
    }

    pub fn watch(
        &mut self,
        paths: Vec<String>,
        recursive: bool,
        ignore_permission_errors: bool,
    ) -> PyResult<()> {
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

    fn _listen_to_events(&self, receiver: Receiver<NotifyResult<NotifyEvent>>) -> PyResult<()> {
        for result in receiver {
            match result {
                Ok(raw_event) => {
                    println!("{:?}", raw_event);

                    let detected_at_ns = get_current_time_ns();

                    if let Some(path_buf) = raw_event.paths.first() {
                        let path = match path_buf.to_str() {
                            Some(s) => s.to_string(),
                            None => {
                                continue;
                            }
                        };

                        let attrs = EventAttributes { tracker: None }; // TODO: fill it with raw_event.attrs info

                        // TODO: find more readable way to remap event data

                        let event = match raw_event.kind {
                            EventKind::Create(create_kind) => match create_kind {
                                CreateKind::File => new_create_event(
                                    Some(ObjectType::File),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                CreateKind::Folder => new_create_event(
                                    Some(ObjectType::Dir),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                CreateKind::Other => new_create_event(
                                    Some(ObjectType::Other),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                CreateKind::Any => {
                                    new_create_event(None, detected_at_ns, path, attrs)
                                }
                            },
                            EventKind::Remove(remove_kind) => match remove_kind {
                                RemoveKind::File => new_remove_event(
                                    Some(ObjectType::File),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                RemoveKind::Folder => new_remove_event(
                                    Some(ObjectType::Dir),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                RemoveKind::Other => new_remove_event(
                                    Some(ObjectType::Other),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                RemoveKind::Any => {
                                    new_remove_event(None, detected_at_ns, path, attrs)
                                }
                            },
                            EventKind::Access(access_kind) => match access_kind {
                                AccessKind::Open(access_mode) => new_access_event(
                                    Some(AccessType::Open),
                                    AccessMode::from_raw(access_mode),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Read => new_access_event(
                                    Some(AccessType::Read),
                                    None,
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Close(access_mode) => new_access_event(
                                    Some(AccessType::Close),
                                    AccessMode::from_raw(access_mode),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Other => new_access_event(
                                    Some(AccessType::Other),
                                    None,
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Any => {
                                    new_access_event(None, None, detected_at_ns, path, attrs)
                                }
                            },
                            EventKind::Modify(modify_kind) => match modify_kind {
                                ModifyKind::Metadata(metadata_kind) => new_modify_metadata_event(
                                    MetadataType::from_raw(metadata_kind),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                ModifyKind::Data(data_changed) => new_modify_data_event(
                                    DataChangeType::from_raw(data_changed),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                ModifyKind::Name(rename_mode) => match rename_mode {
                                    RenameMode::From => new_rename_event(
                                        Some(RenameType::From),
                                        detected_at_ns,
                                        path,
                                        attrs,
                                    ),
                                    RenameMode::To => new_rename_event(
                                        Some(RenameType::To),
                                        detected_at_ns,
                                        path,
                                        attrs,
                                    ),
                                    RenameMode::Both => new_rename_event(
                                        Some(RenameType::Both),
                                        detected_at_ns,
                                        path,
                                        attrs,
                                    ), // TODO: parse the second path
                                    RenameMode::Other => new_rename_event(
                                        Some(RenameType::Other),
                                        detected_at_ns,
                                        path,
                                        attrs,
                                    ),
                                    RenameMode::Any => {
                                        new_rename_event(None, detected_at_ns, path, attrs)
                                    }
                                },
                                ModifyKind::Other => new_modify_event(
                                    Some(ModifyType::Other),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                ModifyKind::Any => {
                                    new_modify_event(None, detected_at_ns, path, attrs)
                                }
                            },
                            EventKind::Other => new_other_event(detected_at_ns, path, attrs),
                            EventKind::Any => new_unknown_event(detected_at_ns, path, attrs),
                        };

                        self.events.lock().unwrap().push_back(event);
                    }
                }
                Err(e) => {
                    // TODO: do something about it
                }
            };
        }

        Ok(())
    }

    pub fn get(&self) -> PyResult<Option<RawEvent>> {
        Ok(self.events.lock().unwrap().pop_front())
    }

    pub fn __enter__(&self, slf: Py<Self>) -> Py<Self> {
        let receiver = self.event_receiver;

        std::thread::Builder::new()
            .name("filesystem_watcher_thread".to_string())
            .spawn(move || self._listen_to_events(receiver))
            .unwrap();

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
