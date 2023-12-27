extern crate notify;
extern crate pyo3;

use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::event::ModifyKind;
use notify::{
    ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;

use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{from_data_kind, from_metadata_kind, ModifyOtherEvent, ModifyUnknownEvent};
use crate::events::rename::from_rename_mode;
use crate::events::EventType;
// use crate::file_cache::FileCache;
use crate::processor::{BatchProcessor, EventProcessor, RawEvent};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    watcher: RecommendedWatcher,
    // file_cache: FileCache,
    processor: Arc<Mutex<BatchProcessor>>, // TODO: use the EventProcessor trait instead
}

impl Watcher {
    pub fn new(buffering_time_ms: u64, debug: bool) -> PyResult<Self> {
        // TODO: hide usage of file cache from Watcher
        // let file_cache = FileCache::new();
        // let file_cache_c = file_cache.clone();

        let processor = Arc::new(Mutex::new(BatchProcessor::new(Duration::from_millis(
            buffering_time_ms,
        ))));

        let processor_c = processor.clone();

        let watcher = RecommendedWatcher::new(
            move |e: Result<Event, notify::Error>| {
                let mut event_processor = processor_c.lock().unwrap();

                if debug {
                    println!("raw event: {:?}", e);
                }

                match e {
                    Ok(e) => event_processor.add_event(e),
                    Err(e) => event_processor.add_error(e),
                }
            },
            notify::Config::default(),
        )
        .unwrap();

        Ok(Watcher {
            debug,
            watcher,
            // file_cache,
            processor,
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

            // self.file_cache.add_root(path, mode);
        }

        if self.debug {
            println!("watcher: {:?}", self.watcher);
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

            // self.file_cache.remove_root(path);
        }

        if self.debug {
            println!("watcher: {:?}", self.watcher);
        }

        Ok(())
    }

    fn create_event(event: &RawEvent) -> Option<EventType> {
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

    pub fn get(&self) -> Vec<EventType> {
        let raw_events;
        let errors;

        {
            let mut processor = self.processor.lock().unwrap();

            raw_events = processor.get_events();
            errors = processor.get_errors();
        }

        if !raw_events.is_empty() && self.debug {
            println!("processed events: {:?}", raw_events);
        }

        if !errors.is_empty() {
            eprintln!("errors: {:?}", errors);
        }

        let mut events: Vec<EventType> = Vec::with_capacity(raw_events.len());

        for raw_event in raw_events {
            if let Some(event) = Self::create_event(&raw_event) {
                events.push(event);
            }
        }

        events
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
}
