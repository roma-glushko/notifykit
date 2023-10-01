extern crate notify;
extern crate pyo3;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use std::io::ErrorKind as IOErrorKind;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use crossbeam_channel::{unbounded, Receiver, RecvError, Sender};

use crate::events::{
    new_access_event, new_create_event, new_modify_data_event, new_modify_event, new_modify_metadata_event,
    new_other_event, new_remove_event, new_rename_event, new_unknown_event, AccessMode, AccessType, DataChangeType,
    EventAttributes, EventType, MetadataType, ModifyType, ObjectType, RawEvent, RenameType,
};
use notify::event::{
    AccessKind, CreateKind, DataChange, Event as NotifyEvent, MetadataKind, ModifyKind, RemoveKind, RenameMode,
};
use notify::{
    Config as NotifyConfig, ErrorKind as NotifyErrorKind, EventKind, PollWatcher, RecommendedWatcher, RecursiveMode,
    Result as NotifyResult, Watcher as NotifyWatcher,
};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

type EventSender = Sender<RawEvent>;
type EventReceiver = Receiver<RawEvent>;
type NotificationReceiver = Receiver<NotifyResult<NotifyEvent>>;

#[derive(Debug)]
enum WatcherType {
    None,
    Poll(PollWatcher),
    Recommended(RecommendedWatcher),
}

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    notification_receiver: NotificationReceiver,
    event_receiver: EventReceiver,
    event_sender: EventSender,
    watcher: WatcherType,
}

impl Watcher {
    pub fn new(debug: bool, force_polling: bool, poll_delay_ms: u64) -> PyResult<Self> {
        if force_polling {
            return Self::new_poll_watcher(debug, poll_delay_ms);
        }

        return Self::new_recommended_watcher(debug, poll_delay_ms);
    }

    fn new_poll_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
        let (notification_sender, notification_receiver) = unbounded();
        let delay = Duration::from_millis(poll_delay_ms);
        let config = NotifyConfig::default().with_poll_interval(delay);

        let watcher = match PollWatcher::new(notification_sender, config) {
            Ok(watcher) => watcher,
            Err(e) => return Err(WatcherError::new_err(format!("Error creating poll watcher: {}", e))),
        };

        let (event_sender, event_receiver) = unbounded::<RawEvent>();

        Ok(Watcher {
            debug,
            notification_receiver,
            event_receiver,
            event_sender,
            watcher: WatcherType::Poll(watcher),
        })
    }

    fn new_recommended_watcher(debug: bool, poll_delay_ms: u64) -> PyResult<Watcher> {
        let (notification_sender, notification_receiver) = unbounded();

        let watcher = match RecommendedWatcher::new(notification_sender, NotifyConfig::default()) {
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

                            return Self::new_poll_watcher(debug, poll_delay_ms);
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

        let (event_sender, event_receiver) = unbounded::<RawEvent>();

        Ok(Watcher {
            debug,
            notification_receiver,
            event_receiver,
            event_sender,
            watcher: WatcherType::Recommended(watcher),
        })
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
                return Err(PyFileNotFoundError::new_err(format!(
                    "No such file or directory: {}",
                    path_str
                )));
            }

            let result = match self.watcher {
                WatcherType::Recommended(ref mut w) => w.watch(path, mode),
                WatcherType::Poll(ref mut w) => w.watch(path, mode),
                WatcherType::None => return Err(WatcherError::new_err("Watcher is closed")),
            };

            match result {
                Err(err) => {
                    if !ignore_permission_errors {
                        return Err(Self::map_notify_error(err));
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
                    return Err(Self::map_notify_error(err));
                }
                _ => (),
            }
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.watcher);
        }

        Ok(())
    }

    fn listen_to_events<'a>(&'a self) -> PyResult<()> {
        for result in &self.notification_receiver {
            match result {
                Ok(raw_event) => {
                    if self.debug {
                        println!("{:?}", raw_event);
                    }

                    let detected_at_ns = Self::get_current_time_ns();

                    if let Some(path_buf) = raw_event.paths.first() {
                        let path = match path_buf.to_str() {
                            Some(s) => s.to_string(),
                            None => {
                                continue;
                            }
                        };

                        // TODO: fill it with raw_event.attrs info
                        let attrs = EventAttributes { tracker: None };

                        // TODO: find more readable way to remap event data
                        let event = match raw_event.kind {
                            EventKind::Create(create_kind) => match create_kind {
                                CreateKind::File => {
                                    new_create_event(Some(ObjectType::File), detected_at_ns, path, attrs)
                                }
                                CreateKind::Folder => {
                                    new_create_event(Some(ObjectType::Dir), detected_at_ns, path, attrs)
                                }
                                CreateKind::Other => {
                                    new_create_event(Some(ObjectType::Other), detected_at_ns, path, attrs)
                                }
                                CreateKind::Any => new_create_event(None, detected_at_ns, path, attrs),
                            },
                            EventKind::Remove(remove_kind) => match remove_kind {
                                RemoveKind::File => {
                                    new_remove_event(Some(ObjectType::File), detected_at_ns, path, attrs)
                                }
                                RemoveKind::Folder => {
                                    new_remove_event(Some(ObjectType::Dir), detected_at_ns, path, attrs)
                                }
                                RemoveKind::Other => {
                                    new_remove_event(Some(ObjectType::Other), detected_at_ns, path, attrs)
                                }
                                RemoveKind::Any => new_remove_event(None, detected_at_ns, path, attrs),
                            },
                            EventKind::Access(access_kind) => match access_kind {
                                AccessKind::Open(access_mode) => new_access_event(
                                    Some(AccessType::Open),
                                    AccessMode::from_raw(access_mode),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Read => {
                                    new_access_event(Some(AccessType::Read), None, detected_at_ns, path, attrs)
                                }
                                AccessKind::Close(access_mode) => new_access_event(
                                    Some(AccessType::Close),
                                    AccessMode::from_raw(access_mode),
                                    detected_at_ns,
                                    path,
                                    attrs,
                                ),
                                AccessKind::Other => {
                                    new_access_event(Some(AccessType::Other), None, detected_at_ns, path, attrs)
                                }
                                AccessKind::Any => new_access_event(None, None, detected_at_ns, path, attrs),
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
                                    RenameMode::From => {
                                        new_rename_event(Some(RenameType::From), detected_at_ns, path, attrs)
                                    }
                                    RenameMode::To => {
                                        new_rename_event(Some(RenameType::To), detected_at_ns, path, attrs)
                                    }
                                    RenameMode::Both => {
                                        new_rename_event(Some(RenameType::Both), detected_at_ns, path, attrs)
                                    } // TODO: parse the second path
                                    RenameMode::Other => {
                                        new_rename_event(Some(RenameType::Other), detected_at_ns, path, attrs)
                                    }
                                    RenameMode::Any => new_rename_event(None, detected_at_ns, path, attrs),
                                },
                                ModifyKind::Other => {
                                    new_modify_event(Some(ModifyType::Other), detected_at_ns, path, attrs)
                                }
                                ModifyKind::Any => new_modify_event(None, detected_at_ns, path, attrs),
                            },
                            EventKind::Other => new_other_event(detected_at_ns, path, attrs),
                            EventKind::Any => new_unknown_event(detected_at_ns, path, attrs),
                        };

                        self.event_sender.send(event).unwrap();
                    }
                }
                Err(e) => {
                    // TODO: do something about it
                    eprintln!("error: {:?}", e);
                }
            };
        }

        Ok(())
    }

    pub fn get(&self) -> PyResult<RawEvent> {
        return Ok(self.event_receiver.recv().unwrap());
    }

    pub fn start(&mut self) {
        let _thread_handle = std::thread::spawn(move || self.listen_to_events());
    }

    pub fn stop(&mut self) {
        self.watcher = WatcherType::None;
    }

    pub fn repr(&self) -> String {
        return format!("Watcher({:#?})", self.watcher);
    }

    fn get_current_time_ns() -> u128 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    fn map_notify_error(notify_error: notify::Error) -> PyErr {
        let err_str = notify_error.to_string();

        match notify_error.kind {
            NotifyErrorKind::PathNotFound => return PyFileNotFoundError::new_err(err_str),
            NotifyErrorKind::Generic(ref err) => {
                // on Windows, we get a Generic with this message when the path does not exist
                if err.as_str() == "Input watch path is neither a file nor a directory." {
                    return PyFileNotFoundError::new_err(err_str);
                }
            }
            NotifyErrorKind::Io(ref io_error) => match io_error.kind() {
                IOErrorKind::NotFound => return PyFileNotFoundError::new_err(err_str),
                IOErrorKind::PermissionDenied => return PyPermissionError::new_err(err_str),
                _ => (),
            },
            _ => (),
        };

        PyOSError::new_err(format!("{} ({:?})", err_str, notify_error))
    }
}
