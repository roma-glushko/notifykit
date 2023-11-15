extern crate notify;
extern crate pyo3;

use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};

use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{from_data_kind, from_metadata_kind, ModifyAnyEvent, ModifyOtherEvent};
use crate::events::others::{OtherEvent, UnknownEvent};
use crate::events::rename::from_rename_mode;
use crate::events::EventType;

use notify::event::{Event as NotifyEvent, ModifyKind};
use notify::{
    Config as NotifyConfig, ErrorKind as NotifyErrorKind, Event, EventKind, PollWatcher, RecommendedWatcher,
    RecursiveMode, Result as NotifyResult, Watcher as NotifyWatcher,
};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

type EventSender = Sender<EventType>;
type EventReceiver = Receiver<EventType>;
type NotificationReceiver = Receiver<NotifyResult<NotifyEvent>>;

#[derive(Debug)]
enum WatcherType {
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
    listen_thread: Option<JoinHandle<()>>,
    stop_listening: Arc<AtomicBool>,
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

        let (event_sender, event_receiver) = unbounded::<EventType>();

        Ok(Watcher {
            debug,
            notification_receiver,
            event_receiver,
            event_sender,
            watcher: WatcherType::Poll(watcher),
            listen_thread: None,
            stop_listening: Arc::new(AtomicBool::new(false)),
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

        let (event_sender, event_receiver) = unbounded::<EventType>();

        Ok(Watcher {
            debug,
            notification_receiver,
            event_receiver,
            event_sender,
            watcher: WatcherType::Recommended(watcher),
            listen_thread: None,
            stop_listening: Arc::new(AtomicBool::new(false)),
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

    fn create_event(notification: &Event) -> EventType {
        let detected_at_ns = Self::get_current_time_ns();

        let paths = &notification.paths;
        let file_path: PathBuf = paths.first().unwrap().to_owned();

        return match notification.kind {
            EventKind::Access(access_kind) => {
                EventType::Access(from_access_kind(detected_at_ns, file_path, access_kind))
            }
            EventKind::Create(create_kind) => {
                EventType::Create(from_create_kind(detected_at_ns, file_path, create_kind))
            }
            EventKind::Remove(delete_kind) => {
                EventType::Delete(from_delete_kind(detected_at_ns, file_path, delete_kind))
            }
            EventKind::Modify(modify_kind) => match modify_kind {
                ModifyKind::Metadata(metadata_kind) => {
                    EventType::ModifyMetadata(from_metadata_kind(detected_at_ns, file_path, metadata_kind))
                }
                ModifyKind::Data(data_kind) => {
                    EventType::ModifyData(from_data_kind(detected_at_ns, file_path, data_kind))
                }
                ModifyKind::Name(rename_mode) => {
                    let target_path = paths.last().to_owned();

                    return EventType::Rename(from_rename_mode(
                        detected_at_ns,
                        file_path,
                        rename_mode,
                        target_path.cloned(),
                    ));
                }
                ModifyKind::Other => EventType::ModifyOther(ModifyOtherEvent::new(detected_at_ns, file_path)),
                ModifyKind::Any => EventType::ModifyAny(ModifyAnyEvent::new(detected_at_ns, file_path)),
            },
            EventKind::Other => EventType::Others(OtherEvent::new(detected_at_ns, file_path)),
            EventKind::Any => EventType::Unknown(UnknownEvent::new(detected_at_ns, file_path)),
        };
    }

    pub fn get(&self, timeout: Duration) -> Result<Option<EventType>, RecvTimeoutError> {
        if self.event_receiver.len() == 0 && self.listen_thread.is_none() {
            return Ok(None);
        }

        return Ok(Some(self.event_receiver.recv_timeout(timeout)?));
    }

    pub fn start(&mut self) {
        let notification_receiver = self.notification_receiver.clone();
        let event_sender = self.event_sender.clone();
        let stop_listening = self.stop_listening.clone();
        let debug = self.debug;

        let listen_thread = std::thread::spawn(move || {
            while !stop_listening.load(Ordering::Relaxed) {
                let timeout = Duration::from_millis(400);
                let timed_out_result = &notification_receiver.recv_timeout(timeout);

                match timed_out_result {
                    Ok(notification_result) => match notification_result {
                        Ok(notification) => {
                            if debug {
                                println!("{:?}", notification);
                            }

                            let raw_event = Self::create_event(notification);

                            event_sender.send(raw_event).unwrap();
                        }
                        Err(e) => {
                            eprintln!("error: {:?}", e);
                        }
                    },
                    Err(e) => match e {
                        RecvTimeoutError::Timeout => (),
                        RecvTimeoutError::Disconnected => {
                            eprintln!("error: {:?}", e);
                        }
                    },
                };
            }
        });

        self.listen_thread = Some(listen_thread)
    }

    pub fn stop(&mut self) {
        if let Some(listen_thread) = self.listen_thread.take() {
            self.stop_listening.store(true, Ordering::Relaxed);

            listen_thread.join().unwrap();
            self.listen_thread = None;
        }
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
