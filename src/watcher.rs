use std::io::ErrorKind as IOErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

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

type TimestampedEvent = (Instant, Result<Event, notify::Error>);
type EventReceiver = mpsc::Receiver<TimestampedEvent>;

#[derive(Debug)]
struct DrainState {
    event_rx: Option<EventReceiver>,
    rx_return: Option<oneshot::Receiver<EventReceiver>>,
    stop_tx: Option<oneshot::Sender<()>>,
    drain_handle: Option<tokio::task::JoinHandle<()>>,
}

impl DrainState {
    fn new(event_rx: EventReceiver) -> Self {
        Self {
            event_rx: Some(event_rx),
            rx_return: None,
            stop_tx: None,
            drain_handle: None,
        }
    }

    fn send_stop_signal(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }

    fn recover_event_rx(&mut self, runtime: &tokio::runtime::Runtime) {
        if let Some(handle) = self.drain_handle.take() {
            // Wait for the task to finish so it can return the receiver.
            // The caller must send the stop signal before calling this.
            let _ = runtime.block_on(handle);
        }

        if let Some(mut rx_return) = self.rx_return.take() {
            match rx_return.try_recv() {
                Ok(rx) => self.event_rx = Some(rx),
                Err(oneshot::error::TryRecvError::Empty) => {
                    // Task hasn't returned the receiver yet; keep the handle for later
                    self.rx_return = Some(rx_return);
                }
                Err(oneshot::error::TryRecvError::Closed) => {
                    // Task dropped the sender (e.g. panic); receiver is lost
                    eprintln!("notifykit: drain task exited without returning event receiver");
                }
            }
        }
    }

    fn stop_and_recover(&mut self, runtime: &tokio::runtime::Runtime) {
        self.send_stop_signal();
        self.recover_event_rx(runtime);
    }

    fn take_event_rx(&mut self) -> Option<EventReceiver> {
        self.event_rx.take()
    }

    fn set_drain(
        &mut self,
        handle: tokio::task::JoinHandle<()>,
        stop_tx: oneshot::Sender<()>,
        rx_return: oneshot::Receiver<EventReceiver>,
    ) {
        self.drain_handle = Some(handle);
        self.stop_tx = Some(stop_tx);
        self.rx_return = Some(rx_return);
    }

}

#[derive(Debug)]
pub(crate) struct Watcher {
    debug: bool,
    event_buffer_size: usize,
    buffering_duration: Duration,
    inner: RecommendedWatcher,
    // file_cache: FileCache,
    drain: DrainState,
    tx: broadcast::Sender<Vec<EventType>>,
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
                if let Err(e) = event_tx.try_send((Instant::now(), e)) {
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
            drain: DrainState::new(event_rx),
            tx,
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
        let runtime = pyo3_async_runtimes::tokio::get_runtime();
        self.drain.stop_and_recover(runtime);

        let (new_tx, _rx) = broadcast::channel::<Vec<EventType>>(self.event_buffer_size);
        self.tx = new_tx;
    }

    pub fn start_drain(&mut self, tick_duration: Duration, event_filter: Option<EventFilter>) {
        let runtime = pyo3_async_runtimes::tokio::get_runtime();
        self.drain.stop_and_recover(runtime);

        let mut event_rx = match self.drain.take_event_rx() {
            Some(rx) => rx,
            None => {
                eprintln!("notifykit: event receiver lost, drain will not start");
                return;
            }
        };

        let (stop_tx, mut stop_rx) = oneshot::channel();
        let (rx_return_tx, rx_return_rx) = oneshot::channel();

        let tx = self.tx.clone();
        let debug = self.debug;
        let buffering_duration = self.buffering_duration;

        let handle = runtime.spawn(async move {
            let mut processor = BatchProcessor::new(buffering_duration);
            let mut ticker = time::interval(tick_duration);

            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = ticker.tick() => {
                        while let Ok((time, result)) = event_rx.try_recv() {
                            match result {
                                Ok(event) => processor.add_event(event, time),
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
        });

        self.drain.set_drain(handle, stop_tx, rx_return_rx);
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

#[cfg(test)]
mod tests {
    use super::*;
    use notify::Event as NotifyEvent;

    fn build_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    /// Helper: spawn a minimal drain task that moves the receiver in
    /// and returns it via oneshot when the stop signal is received.
    fn spawn_drain(
        runtime: &tokio::runtime::Runtime,
        drain: &mut DrainState,
    ) {
        let mut event_rx = drain.take_event_rx().expect("event_rx should be available");

        let (stop_tx, mut stop_rx) = oneshot::channel();
        let (rx_return_tx, rx_return_rx) = oneshot::channel();

        let handle = runtime.spawn(async move {
            // Minimal drain loop: just wait for stop signal
            loop {
                tokio::select! {
                    _ = &mut stop_rx => break,
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // drain events to keep channel from filling
                        while event_rx.try_recv().is_ok() {}
                    }
                }
            }
            let _ = rx_return_tx.send(event_rx);
        });

        drain.set_drain(handle, stop_tx, rx_return_rx);
    }

    #[test]
    fn new_drain_state_has_receiver() {
        let (_tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let drain = DrainState::new(rx);

        assert!(drain.event_rx.is_some());
        assert!(drain.rx_return.is_none());
        assert!(drain.stop_tx.is_none());
        assert!(drain.drain_handle.is_none());
    }

    #[test]
    fn stop_and_recover_returns_receiver() {
        let rt = build_runtime();
        let (_tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        // Start a drain task
        spawn_drain(&rt, &mut drain);
        assert!(drain.event_rx.is_none(), "receiver should be moved into task");

        // Stop and recover
        drain.stop_and_recover(&rt);
        assert!(drain.event_rx.is_some(), "receiver should be recovered after stop");
    }

    #[test]
    fn stop_and_recover_without_active_drain_is_noop() {
        let rt = build_runtime();
        let (_tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        // No drain running — should not panic
        drain.stop_and_recover(&rt);
        assert!(drain.event_rx.is_some());
    }

    #[test]
    fn multiple_stop_restart_cycles() {
        let rt = build_runtime();
        let (tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        for i in 0..5 {
            spawn_drain(&rt, &mut drain);
            assert!(drain.event_rx.is_none(), "cycle {i}: receiver should be in task");

            // Send an event while drain is running
            let event = NotifyEvent::default();
            tx.try_send((Instant::now(), Ok(event))).ok();

            drain.stop_and_recover(&rt);
            assert!(
                drain.event_rx.is_some(),
                "cycle {i}: receiver should be recovered"
            );
        }
    }

    #[test]
    fn events_survive_stop_restart_cycle() {
        let rt = build_runtime();
        let (tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        // Start and stop drain
        spawn_drain(&rt, &mut drain);
        drain.stop_and_recover(&rt);

        // Send event after recovery
        let event = NotifyEvent::default();
        tx.try_send((Instant::now(), Ok(event))).unwrap();

        // Receiver should still work
        let event_rx = drain.event_rx.as_mut().unwrap();
        let received = event_rx.try_recv();
        assert!(received.is_ok(), "should receive event after stop/restart");
    }

    #[test]
    fn channel_stays_connected_after_recovery() {
        let rt = build_runtime();
        let (tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        // Multiple cycles with events in between
        for _ in 0..3 {
            spawn_drain(&rt, &mut drain);
            drain.stop_and_recover(&rt);
        }

        // The sender should still be connected to the recovered receiver
        let event = NotifyEvent::default();
        assert!(
            tx.try_send((Instant::now(), Ok(event))).is_ok(),
            "sender should still be connected after multiple cycles"
        );

        let event_rx = drain.event_rx.as_mut().unwrap();
        assert!(event_rx.try_recv().is_ok());
    }

    #[test]
    fn recover_handles_task_that_dropped_return_sender() {
        let rt = build_runtime();
        let (_tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        let event_rx = drain.take_event_rx().unwrap();

        let (stop_tx, mut stop_rx) = oneshot::channel();
        let (rx_return_tx, rx_return_rx) = oneshot::channel();

        // Spawn a task that drops the return sender without sending
        let handle = rt.spawn(async move {
            let _rx = event_rx; // take ownership
            let _ = &mut stop_rx;
            drop(rx_return_tx); // simulate panic/early exit
        });

        drain.set_drain(handle, stop_tx, rx_return_rx);
        drain.stop_and_recover(&rt);

        // Receiver is lost — this is the expected failure mode
        assert!(
            drain.event_rx.is_none(),
            "receiver should be lost when task drops return sender"
        );
    }

    #[test]
    fn send_stop_signal_is_idempotent() {
        let (_tx, rx) = mpsc::channel::<TimestampedEvent>(16);
        let mut drain = DrainState::new(rx);

        // No stop_tx set — should not panic
        drain.send_stop_signal();
        drain.send_stop_signal();
    }
}
