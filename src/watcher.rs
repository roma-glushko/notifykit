extern crate notify;
extern crate pyo3;

use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{from_data_kind, from_metadata_kind, ModifyOtherEvent, ModifyUnknownEvent};
use crate::events::rename::from_rename_mode;
use crate::events::EventType;
use notify::event::ModifyKind;
use notify::{
    ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use tokio::{
    sync::{broadcast, oneshot},
    time,
};
// use crate::file_cache::FileCache;
use crate::processor::{BatchProcessor, EventProcessor, RawEvent};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    inner: RecommendedWatcher,
    // file_cache: FileCache,
    processor: Arc<Mutex<BatchProcessor>>, // TODO: use the EventProcessor trait instead
    tx: broadcast::Sender<Vec<EventType>>,
    stop_tx: Option<oneshot::Sender<()>>,
}

impl Watcher {
    pub fn new(buffering_time_ms: u64, debug: bool) -> Result<Self, notify::Error> {
        // TODO: hide usage of file cache from Watcher
        // let file_cache = FileCache::new();
        // let file_cache_c = file_cache.clone();

        let buffering_duration = Duration::from_millis(buffering_time_ms);
        let processor = Arc::new(Mutex::new(BatchProcessor::new(buffering_duration)));
        let processor_c = processor.clone();

        let (tx, _rx) = broadcast::channel::<Vec<EventType>>(1024); // TODO: make buffer size configurable

        let inner = RecommendedWatcher::new(
            move |e: Result<Event, notify::Error>| {
                let mut event_processor = match processor_c.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        eprintln!("notifykit: event processor lock poisoned, dropping event: {e}");
                        return;
                    }
                };

                if debug {
                    println!("raw event: {:?}", e);
                }

                match e {
                    Ok(e) => event_processor.add_event(e),
                    Err(e) => event_processor.add_error(e),
                }
            },
            notify::Config::default(),
        )?;

        Ok(Self {
            debug,
            inner,
            processor,
            tx,
            stop_tx: None,
        })
    }

    pub fn watch(&mut self, paths: &[String], recursive: bool, ignore_perm: bool) -> PyResult<()> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for p in paths {
            let path = PathBuf::from(&p);

            if !path.exists() {
                return Err(PyFileNotFoundError::new_err(format!(
                    "No such file or directory: {}",
                    p
                )));
            }

            let result = self.inner.watch(&path, mode);

            if let Err(err) = result {
                if !ignore_perm {
                    return Err(map_notify_error(err));
                }
            }

            // self.file_cache.add_root(path, mode);
        }

        if self.debug {
            println!("watcher: {:?}", self.inner);
        }

        Ok(())
    }

    pub fn unwatch(&mut self, paths: Vec<String>) -> PyResult<()> {
        for path_str in paths.into_iter() {
            let path = Path::new(&path_str);

            let result = self.inner.unwatch(path);

            if let Err(err) = result {
                return Err(map_notify_error(err));
            }

            // self.file_cache.remove_root(path);
        }

        if self.debug {
            println!("watcher: {:?}", self.inner);
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }

        let (new_tx, _rx) = broadcast::channel::<Vec<EventType>>(1024);
        let _old = std::mem::replace(&mut self.tx, new_tx);
    }

    pub fn start_drain(&mut self, debounce_delay: Duration) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }

        let (stop_tx, mut stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let proc = Arc::clone(&self.processor);
        let tx = self.tx.clone();
        let debug = self.debug;

        pyo3_asyncio::tokio::get_runtime().spawn(async move {
            let mut ticker = time::interval(debounce_delay);

            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = ticker.tick() => {
                        let (raw, errs) = {
                            let mut p = match proc.lock() {
                                Ok(guard) => guard,
                                Err(e) => {
                                    eprintln!("notifykit: event processor lock poisoned, skipping drain tick: {e}");
                                    continue;
                                }
                            };
                            (p.get_events(), p.get_errors())
                        };
                        if debug && !raw.is_empty() { println!("processed: {:?}", raw); }
                        if !errs.is_empty() { eprintln!("errors: {:?}", errs); }
                        if raw.is_empty() { continue; }

                        let mut batch = Vec::with_capacity(raw.len());
                        for r in raw {
                            if let Some(ev) = create_event(&r) { batch.push(ev); }
                        }

                        if !batch.is_empty() { let _ = tx.send(batch); }
                    }
                }
            }
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Vec<EventType>> {
        self.tx.subscribe()
    }

    pub fn repr(&mut self) -> String {
        format!("Watcher({:#?})", self.inner)
    }
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

fn create_event(event: &RawEvent) -> Option<EventType> {
    let paths = &event.paths;
    let file_path: PathBuf = paths.first()?.to_owned();

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
