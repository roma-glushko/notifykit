extern crate notify;
extern crate pyo3;

use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use notify::event::ModifyKind;
use notify::{ErrorKind as NotifyErrorKind, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;

use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{from_data_kind, from_metadata_kind, ModifyAnyEvent, ModifyOtherEvent};
use crate::events::rename::from_rename_mode;
use crate::events::EventType;

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

type EventSender = Sender<EventType>;
type EventReceiver = Receiver<EventType>;
type NotificationReceiver = Receiver<DebounceEventResult>;

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    notification_receiver: NotificationReceiver,
    event_receiver: EventReceiver,
    event_sender: EventSender,
    debouncer: Debouncer<RecommendedWatcher, FileIdMap>,
    listen_thread: Option<JoinHandle<()>>,
    stop_listening: Arc<AtomicBool>,
}

impl Watcher {
    pub fn new(debounce_ms: u64, debounce_tick_rate_ms: Option<u64>, debug: bool) -> PyResult<Self> {
        let (notification_sender, notification_receiver) = unbounded();

        // The logic that debouncer incorporates is the heart of notifykit,
        // so it's possible we may need to take under the umbrella of
        // the project to further improve if needed
        let result = new_debouncer(
            Duration::from_millis(debounce_ms),
            debounce_tick_rate_ms.map(Duration::from_millis),
            notification_sender,
        ); // TODO: handle this error

        let debouncer = match result {
            Ok(debouncer) => debouncer,
            Err(e) => return Err(WatcherError::new_err(format!("Error creating poll watcher: {}", e))),
        };

        let (event_sender, event_receiver) = unbounded::<EventType>();

        Ok(Watcher {
            debug,
            notification_receiver,
            event_receiver,
            event_sender,
            debouncer,
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

            let result = self.debouncer.watcher().watch(path, mode);

            if let Err(err) = result {
                if !ignore_permission_errors {
                    return Err(Self::map_notify_error(err));
                }
            }

            self.debouncer.cache().add_root(path, mode);
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.debouncer.watcher());
        }

        Ok(())
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        for path_str in paths.into_iter() {
            let path = Path::new(&path_str);

            let result = self.debouncer.watcher().unwatch(path);

            if let Err(err) = result {
                return Err(Self::map_notify_error(err));
            }

            self.debouncer.cache().remove_root(path);
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.debouncer.watcher());
        }

        Ok(())
    }

    fn create_event(event: &DebouncedEvent) -> Option<EventType> {
        let paths = &event.paths;
        let file_path: PathBuf = paths.first().unwrap().to_owned();

        Some(match event.kind {
            EventKind::Access(access_kind) => EventType::Access(from_access_kind(file_path, access_kind)),
            EventKind::Create(create_kind) => EventType::Create(from_create_kind(file_path, create_kind)),
            EventKind::Remove(delete_kind) => EventType::Delete(from_delete_kind(file_path, delete_kind)),
            EventKind::Modify(modify_kind) => match modify_kind {
                ModifyKind::Metadata(metadata_kind) => {
                    EventType::ModifyMetadata(from_metadata_kind(file_path, metadata_kind))
                }
                ModifyKind::Data(data_kind) => EventType::ModifyData(from_data_kind(file_path, data_kind)),
                ModifyKind::Name(_) => {
                    // Debouncer stitches rename events, so rename_kind is not relevant
                    let target_path = paths.last().cloned()?;

                    return Some(EventType::Rename(from_rename_mode(file_path, target_path)));
                }
                ModifyKind::Other => EventType::ModifyOther(ModifyOtherEvent::new(file_path)),
                ModifyKind::Any => EventType::ModifyAny(ModifyAnyEvent::new(file_path)),
            },
            EventKind::Other | EventKind::Any => {
                // Debouncer ignores these events, so we are not going to receive them
                return None;
            }
        })
    }

    pub fn get(&self, timeout: Duration) -> Result<Option<EventType>, RecvTimeoutError> {
        if self.event_receiver.is_empty() && self.listen_thread.is_none() {
            return Ok(None);
        }

        Ok(Some(self.event_receiver.recv_timeout(timeout)?))
    }

    pub fn start(&mut self, tick_rate_ms: u64) {
        let notification_receiver = self.notification_receiver.clone();
        let event_sender = self.event_sender.clone();
        let stop_listening = self.stop_listening.clone();
        let debug = self.debug;

        let listen_thread = std::thread::spawn(move || {
            while !stop_listening.load(Ordering::Relaxed) {
                let timeout = Duration::from_millis(tick_rate_ms);
                let timed_out_result = &notification_receiver.recv_timeout(timeout);

                match timed_out_result {
                    Ok(notification_result) => match notification_result {
                        Ok(notifications) => {
                            if debug {
                                println!("{:?}", notifications);
                            }

                            for raw_notification in notifications {
                                if let Some(event) = Self::create_event(raw_notification) {
                                    event_sender.send(event).unwrap();
                                }
                            }
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
            // self.debouncer.stop();

            self.stop_listening.store(true, Ordering::Relaxed);

            listen_thread.join().unwrap();
            self.listen_thread = None;
        }
    }

    pub fn repr(&mut self) -> String {
        format!("Watcher({:#?})", self.debouncer.watcher())
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
