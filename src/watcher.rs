use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::events::EventType;
use crate::events::access::from_access_kind;
use crate::events::create::from_create_kind;
use crate::events::delete::from_delete_kind;
use crate::events::modify::{ModifyOtherEvent, ModifyUnknownEvent, from_data_kind, from_metadata_kind};
use crate::events::rename::from_rename_mode;
use notify::event::ModifyKind;
use notify::{
    ErrorKind as NotifyErrorKind, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use pyo3::exceptions::{PyException, PyFileNotFoundError, PyOSError, PyPermissionError};
use pyo3::prelude::*;
use tokio::{
    sync::{broadcast, mpsc, oneshot},
    time,
};
// use crate::file_cache::FileCache;
use crate::filter::EventFilter;
use crate::processor::{BatchProcessor, EventProcessor, RawEvent};

pyo3::create_exception!(_inotify_toolkit_lib, WatcherError, PyException);

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    event_buffer_size: usize,
    buffering_duration: Duration,
    inner: RecommendedWatcher,
    // file_cache: FileCache,
    event_rx: Option<mpsc::Receiver<Result<Event, notify::Error>>>,
    rx_return: Option<oneshot::Receiver<mpsc::Receiver<Result<Event, notify::Error>>>>,
    tx: broadcast::Sender<Vec<EventType>>,
    stop_tx: Option<oneshot::Sender<()>>,
    drain_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Watcher {
    pub fn new(
        buffering_time_ms: u64,
        event_buffer_size: usize,
        debug: bool,
        follow_symlinks: bool,
    ) -> Result<Self, notify::Error> {
        let buffering_duration = Duration::from_millis(buffering_time_ms);
        let (event_tx, event_rx) = mpsc::channel(event_buffer_size);

        let (tx, _rx) = broadcast::channel::<Vec<EventType>>(event_buffer_size);

        let config = notify::Config::default().with_follow_symlinks(follow_symlinks);

        let inner = RecommendedWatcher::new(
            move |e: Result<Event, notify::Error>| {
                if debug {
                    println!("raw event: {:?}", e);
                }
                if let Err(e) = event_tx.try_send(e) {
                    eprintln!("event channel full or closed, dropping event: {e}");
                }
            },
            config,
        )?;

        Ok(Self {
            debug,
            event_buffer_size,
            buffering_duration,
            inner,
            event_rx: Some(event_rx),
            rx_return: None,
            tx,
            stop_tx: None,
            drain_handle: None,
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

        self.recover_event_rx();

        let (new_tx, _rx) = broadcast::channel::<Vec<EventType>>(self.event_buffer_size);
        self.tx = new_tx;
    }

    fn recover_event_rx(&mut self) {
        if let Some(handle) = self.drain_handle.take() {
            handle.abort();
        }

        if let Some(mut rx_return) = self.rx_return.take() {
            if let Ok(rx) = rx_return.try_recv() {
                self.event_rx = Some(rx);
            }
        }
    }

    pub fn start_drain(&mut self, tick_duration: Duration, event_filter: Option<EventFilter>) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }

        self.recover_event_rx();

        let mut event_rx = match self.event_rx.take() {
            Some(rx) => rx,
            None => return,
        };

        let (stop_tx, mut stop_rx) = oneshot::channel();
        self.stop_tx = Some(stop_tx);

        let (rx_return_tx, rx_return_rx) = oneshot::channel();
        self.rx_return = Some(rx_return_rx);

        let tx = self.tx.clone();
        let debug = self.debug;
        let buffering_duration = self.buffering_duration;

        self.drain_handle = Some(pyo3_async_runtimes::tokio::get_runtime().spawn(async move {
            let mut processor = BatchProcessor::new(buffering_duration);
            let mut ticker = time::interval(tick_duration);

            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = ticker.tick() => {
                        while let Ok(result) = event_rx.try_recv() {
                            match result {
                                Ok(event) => processor.add_event(event),
                                Err(error) => processor.add_error(error),
                            }
                        }

                        let raw = processor.get_events();
                        let errs = processor.get_errors();

                        if debug && !raw.is_empty() { println!("processed: {:?}", raw); }
                        if !errs.is_empty() { eprintln!("errors: {:?}", errs); }
                        if raw.is_empty() { continue; }

                        let mut batch = Vec::with_capacity(raw.len());
                        for r in raw {
                            if let Some(ev) = create_event(&r) {
                                if let Some(ref filter) = event_filter {
                                    if !filter.should_filter(&ev) {
                                        batch.push(ev);
                                    }
                                } else {
                                    batch.push(ev);
                                }
                            }
                        }

                        if !batch.is_empty() {
                            if let Err(e) = tx.send(batch) {
                                eprintln!("failed to broadcast events: {e}");
                            }
                        }
                    }
                }
            }

            let _ = rx_return_tx.send(event_rx);
        }));
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
