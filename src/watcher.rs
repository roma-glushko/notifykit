extern crate notify;
extern crate pyo3;

use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use notify::event::ModifyKind;
use notify::{Error, ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, FileIdMap};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;

use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{from_data_kind, from_metadata_kind, ModifyOtherEvent, ModifyUnknownEvent};
use crate::events::rename::from_rename_mode;
use crate::events::EventType;
use crate::processor::EventProcessor;

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

type EventSender = Sender<EventType>;
type EventReceiver = Receiver<EventType>;
type NotificationReceiver = Receiver<DebounceEventResult>;

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    watcher: RecommendedWatcher,
    file_cache: FileIdMap,
    processor: Arc<Mutex<EventProcessor<FileIdMap>>>,
    listen_thread: Option<JoinHandle<()>>,
    stop_listening: Arc<AtomicBool>,
}

impl Watcher {
    pub fn new(buffering_time_ms: u64, debug: bool) -> PyResult<Self> {
        let file_cache =  FileIdMap::new();
        let file_cache_c = file_cache.clone();

        let processor = Arc::new(Mutex::new(EventProcessor::new(
            file_cache,
            Duration::from_millis(buffering_time_ms)
        )));

        let processor_c = processor.clone();

        let watcher = RecommendedWatcher::new(
            move |e: Result<Event, Error>| {
                let mut event_processor = processor_c.lock();

                match e {
                    Ok(e) => event_processor.add_event(e),
                    Err(e) => event_processor.add_error(e),
                }
            },
            notify::Config::default(),
        )?;

        Ok(Watcher {
            debug,
            watcher,
            file_cache: file_cache_c,
            processor,
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

            let result = self.watcher.watch(path, mode);

            if let Err(err) = result {
                if !ignore_permission_errors {
                    return Err(Self::map_notify_error(err));
                }
            }

            self.file_cache.add_root(path, mode);
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.watcher);
        }

        Ok(())
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        for path_str in paths.into_iter() {
            let path = Path::new(&path_str);

            let result = self.watcher.unwatch(path);

            if let Err(err) = result {
                return Err(Self::map_notify_error(err));
            }

            self.file_cache.remove_root(path);
        }

        if self.debug {
            eprintln!("watcher: {:?}", self.watcher);
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
                ModifyKind::Any => EventType::ModifyUnknown(ModifyUnknownEvent::new(file_path)),
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

    pub fn start(&mut self, tick_rate_ms: Option<u64>) {
        // let notification_receiver = self.notification_receiver.clone();
        // let event_sender = self.event_sender.clone();
        let processor_c = self.processor.clone()
        let stop_listening = self.stop_listening.clone();
        let debug = self.debug;

        let tick_rate = self.get_tick_rate(tick_rate_ms)?;

        let listen_thread = std::thread::Builder::new()
            .name("notifykit watcher loop".to_string())
            .spawn(move || {
            while !stop_listening.load(Ordering::Acquire) {
                std::thread::sleep(tick_rate);

                let raw_events;
                let errors;

                {
                    let mut processor = processor_c.lock();

                    raw_events = processor.debounced_events();
                    errors = processor.errors();
                }

                if debug {
                    println!("{:?}", notifications);
                }

                for raw_notification in notifications {
                    if let Some(event) = Self::create_event(raw_notification) {
                        event_sender.send(event)?;
                    }
                }

                if !raw_events.is_empty() {
                    event_handler.handle_event(Ok(send_data));
                }

                if !errors.is_empty() {
                    event_handler.handle_event(Err(errors));
                }
            }
        })?;

        self.listen_thread = Some(listen_thread)
    }

    pub fn set_stop(&mut self) {
        self.stop_listening.store(true, Ordering::Relaxed);
    }

    pub fn stop(&mut self) {
        if let Some(listen_thread) = self.listen_thread.take() {
            // self.debouncer.stop();

            self.set_stop();

            listen_thread.join().unwrap();
            self.listen_thread = None;
        }
    }

    pub fn repr(&mut self) -> String {
        format!("Watcher({:#?})", self.watcher)
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

    fn get_tick_rate(tick_rate_ms: Option<u64>) -> {
        let tick_div = 5;

        let tick = match tick_rate {
            Some(v) => {
                if v > timeout {
                    return Err(Error::new(ErrorKind::Generic(format!(
                        "Invalid tick_rate, tick rate {:?} > {:?} timeout!",
                        v, timeout
                    ))));
                }
                v
            }
            None => timeout.checked_div(tick_div).ok_or_else(|| {
                Error::new(ErrorKind::Generic(format!(
                    "Failed to calculate tick as {:?}/{}!",
                    timeout, tick_div
                )))
            })?,
        };

    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        self.set_stop();
    }
}
