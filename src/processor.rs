use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::Duration;

#[cfg(not(test))]
use std::time::Instant;

use crate::file_cache::FileIdCache;
use file_id::FileId;
use notify::event::{ModifyKind, RemoveKind, RenameMode};
use notify::{Error as NotifyError, Event as NotifyEvent, EventKind};

/// A debounced event is emitted after a short delay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEvent {
    /// The original event.
    pub event: NotifyEvent,

    /// The time at which the event occurred.
    pub time: Instant,
}

impl RawEvent {
    pub fn new(event: NotifyEvent, time: Instant) -> Self {
        Self { event, time }
    }
}

impl Deref for RawEvent {
    type Target = NotifyEvent;

    fn deref(&self) -> &Self::Target {
        &self.event
    }
}

impl DerefMut for RawEvent {
    fn deref_mut(&mut self) -> &mut NotifyEvent {
        &mut self.event
    }
}

impl Default for RawEvent {
    fn default() -> Self {
        Self {
            event: Default::default(),
            time: Instant::now(),
        }
    }
}

impl From<NotifyEvent> for RawEvent {
    fn from(event: NotifyEvent) -> Self {
        Self {
            event,
            time: Instant::now(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct FileEventQueue {
    /// Events must be stored in the following order:
    /// 1. `remove` or `move out` event
    /// 2. `rename` event
    /// 3. Other events
    events: VecDeque<RawEvent>,
}

impl FileEventQueue {
    fn was_created(&self) -> bool {
        self.events.front().map_or(false, |event| {
            matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(ModifyKind::Name(RenameMode::To))
            )
        })
    }

    fn was_removed(&self) -> bool {
        self.events.front().map_or(false, |event| {
            matches!(
                event.kind,
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From))
            )
        })
    }
}

pub(crate) trait EventProcessor {
    fn get_events(&mut self) -> Vec<RawEvent>;
    fn get_errors(&mut self) -> Vec<NotifyError>;
    fn add_event(&mut self, event: NotifyEvent);
    fn add_error(&mut self, error: NotifyError);
}

#[derive(Debug)]
pub struct CrossPlatformEventProcessor<T> {
    events_by_file: HashMap<PathBuf, FileEventQueue>,
    file_cache: T,
    rename_event: Option<(RawEvent, Option<FileId>)>,
    rescan_event: Option<RawEvent>,
    errors: Vec<NotifyError>,
    buffering_time: Duration,
}

// TODO: allow to specify cross-platform event processor when it's bugs are fixed
#[allow(unused)]
impl<T: FileIdCache> CrossPlatformEventProcessor<T> {
    pub fn new(file_cache: T, buffering_time: Duration) -> Self {
        Self {
            events_by_file: HashMap::new(),
            file_cache,
            rename_event: None,
            rescan_event: None,
            errors: Vec::new(),
            buffering_time,
        }
    }

    fn handle_rename_from(&mut self, event: NotifyEvent) {
        let time = Instant::now();
        let path = &event.paths[0];

        // store event
        let file_id = self.file_cache.get_file_id(path).cloned();
        self.rename_event = Some((RawEvent::new(event.clone(), time), file_id));

        self.file_cache.remove_path(path);

        self.push_event(event, time);
    }

    fn handle_rename_to(&mut self, event: NotifyEvent) {
        self.file_cache.add_path(&event.paths[0]);

        let trackers_match = self
            .rename_event
            .as_ref()
            .and_then(|(e, _)| e.tracker())
            .and_then(|from_tracker| event.attrs.tracker().map(|to_tracker| from_tracker == to_tracker))
            .unwrap_or_default();

        let file_ids_match = self
            .rename_event
            .as_ref()
            .and_then(|(_, id)| id.as_ref())
            .and_then(|from_file_id| {
                self.file_cache
                    .get_file_id(&event.paths[0])
                    .map(|to_file_id| from_file_id == to_file_id)
            })
            .unwrap_or_default();

        if trackers_match || file_ids_match {
            // connect rename
            let (mut rename_event, _) = self.rename_event.take().unwrap(); // unwrap is safe because `rename_event` must be set at this point
            let path = rename_event.paths.remove(0);
            let time = rename_event.time;
            self.push_rename_event(path, event, time);
        } else {
            // move in
            self.push_event(event, Instant::now());
        }

        self.rename_event = None;
    }

    fn push_rename_event(&mut self, path: PathBuf, event: NotifyEvent, time: Instant) {
        self.file_cache.remove_path(&path);

        let mut source_queue = self.events_by_file.remove(&path).unwrap_or_default();

        // remove rename `from` event
        source_queue.events.pop_back();

        // remove existing rename event
        let (remove_index, original_path, original_time) = source_queue
            .events
            .iter()
            .enumerate()
            .find_map(|(index, e)| {
                if matches!(e.kind, EventKind::Modify(ModifyKind::Name(RenameMode::Both))) {
                    Some((Some(index), e.paths[0].clone(), e.time))
                } else {
                    None
                }
            })
            .unwrap_or((None, path, time));

        if let Some(remove_index) = remove_index {
            source_queue.events.remove(remove_index);
        }

        // split off remove or move out event and add it back to the events map
        if source_queue.was_removed() {
            let event = source_queue.events.pop_front().unwrap();

            self.events_by_file
                .insert(event.paths[0].clone(), FileEventQueue { events: [event].into() });
        }

        // update paths
        for e in &mut source_queue.events {
            e.paths = vec![event.paths[0].clone()];
        }

        // insert rename event at the front, unless the file was just created
        if !source_queue.was_created() {
            source_queue.events.push_front(RawEvent {
                event: NotifyEvent {
                    kind: EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
                    paths: vec![original_path, event.paths[0].clone()],
                    attrs: event.attrs,
                },
                time: original_time,
            });
        }

        if let Some(target_queue) = self.events_by_file.get_mut(&event.paths[0]) {
            if !target_queue.was_created() {
                let mut remove_event = RawEvent {
                    event: NotifyEvent {
                        kind: EventKind::Remove(RemoveKind::Any),
                        paths: vec![event.paths[0].clone()],
                        attrs: Default::default(),
                    },
                    time: original_time,
                };
                if !target_queue.was_removed() {
                    remove_event.event = remove_event.event.set_info("override");
                }
                source_queue.events.push_front(remove_event);
            }
            *target_queue = source_queue;
        } else {
            self.events_by_file.insert(event.paths[0].clone(), source_queue);
        }
    }

    fn push_remove_event(&mut self, event: NotifyEvent, time: Instant) {
        let path = &event.paths[0];

        // remove child queues
        self.events_by_file.retain(|p, _| !p.starts_with(path) || p == path);

        // remove cached file ids
        self.file_cache.remove_path(path);

        match self.events_by_file.get_mut(path) {
            Some(queue) if queue.was_created() => {
                self.events_by_file.remove(path);
            }
            Some(queue) => {
                queue.events = [RawEvent::new(event, time)].into();
            }
            None => {
                self.push_event(event, time);
            }
        }
    }

    fn push_event(&mut self, event: NotifyEvent, time: Instant) {
        let path = &event.paths[0];

        if let Some(queue) = self.events_by_file.get_mut(path) {
            // skip duplicate create events and modifications right after creation
            if match event.kind {
                EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Metadata(_)) | EventKind::Create(_) => {
                    !queue.was_created()
                }
                _ => true,
            } {
                queue.events.push_back(RawEvent::new(event, time));
            }
        } else {
            self.events_by_file.insert(
                path.to_path_buf(),
                FileEventQueue {
                    events: [RawEvent::new(event, time)].into(),
                },
            );
        }
    }
}

impl<T: FileIdCache> EventProcessor for CrossPlatformEventProcessor<T> {
    fn get_events(&mut self) -> Vec<RawEvent> {
        let now = Instant::now();
        let mut events_to_return = Vec::with_capacity(self.events_by_file.len());
        let mut remaining_events = HashMap::with_capacity(self.events_by_file.len());

        if let Some(rescan_event) = self.rescan_event.take() {
            if now.saturating_duration_since(rescan_event.time) >= self.buffering_time {
                // log::trace!("debounced event: {rescan_event:?}");

                events_to_return.push(rescan_event);
            } else {
                self.rescan_event = Some(rescan_event);
            }
        }

        // TODO: perfect fit for drain_filter https://github.com/rust-lang/rust/issues/59618
        for (path, mut events) in self.events_by_file.drain() {
            let mut kind_index = HashMap::new();

            while let Some(event) = events.events.pop_front() {
                if now.saturating_duration_since(event.time) >= self.buffering_time {
                    // remove previous event of the same kind
                    if let Some(idx) = kind_index.get(&event.kind).copied() {
                        events_to_return.remove(idx);

                        kind_index.values_mut().for_each(|i| {
                            if *i > idx {
                                *i -= 1
                            }
                        })
                    }

                    kind_index.insert(event.kind, events_to_return.len());

                    events_to_return.push(event);
                } else {
                    events.events.push_front(event);
                    break;
                }
            }

            if !events.events.is_empty() {
                remaining_events.insert(path, events);
            }
        }

        self.events_by_file = remaining_events;

        // order events for different files chronologically, but keep the order of events for the same file
        events_to_return.sort_by(|event_a, event_b| {
            // use the last path because rename events are emitted for the target path
            if event_a.paths.last() == event_b.paths.last() {
                std::cmp::Ordering::Equal
            } else {
                event_a.time.cmp(&event_b.time)
            }
        });

        events_to_return
    }

    /// Returns all currently stored errors
    fn get_errors(&mut self) -> Vec<NotifyError> {
        let mut v = Vec::new();
        std::mem::swap(&mut v, &mut self.errors);
        v
    }

    /// Add new event to debouncer cache
    fn add_event(&mut self, event: NotifyEvent) {
        // log::trace!("raw event: {event:?}");

        if event.need_rescan() {
            self.file_cache.rescan();
            self.rescan_event = Some(event.into());
            return;
        }

        let path = &event.paths[0];

        match &event.kind {
            EventKind::Create(_) => {
                self.file_cache.add_path(path);

                self.push_event(event, Instant::now());
            }
            EventKind::Modify(ModifyKind::Name(rename_mode)) => {
                match rename_mode {
                    RenameMode::Any => {
                        if event.paths[0].exists() {
                            self.handle_rename_to(event);
                        } else {
                            self.handle_rename_from(event);
                        }
                    }
                    RenameMode::To => {
                        self.handle_rename_to(event);
                    }
                    RenameMode::From => {
                        self.handle_rename_from(event);
                    }
                    RenameMode::Both => {
                        // ignore and handle `To` and `From` events instead
                    }
                    RenameMode::Other => {
                        // unused
                    }
                }
            }
            EventKind::Remove(_) => {
                self.push_remove_event(event, Instant::now());
            }
            EventKind::Other => {
                // ignore meta events
            }
            _ => {
                if self.file_cache.get_file_id(path).is_none() {
                    self.file_cache.add_path(path);
                }

                self.push_event(event, Instant::now());
            }
        }
    }

    /// Add an error entry to re-send later on
    fn add_error(&mut self, error: NotifyError) {
        self.errors.push(error);
    }
}

#[derive(Debug)]
pub struct BatchProcessor {
    buffering_time: Duration,
    events: Vec<RawEvent>,
    errors: Vec<NotifyError>,
}

/// A simple event processor that buffers incoming events for a given amount
/// of time without any modification to the underlying events
impl BatchProcessor {
    pub fn new(buffering_time: Duration) -> Self {
        Self {
            events: Vec::new(),
            errors: Vec::new(),
            buffering_time,
        }
    }
}

impl EventProcessor for BatchProcessor {
    fn get_events(&mut self) -> Vec<RawEvent> {
        let now = Instant::now();

        // Assuming expired items are contiguous and at the beginning of the vector
        let first_expired_index = self
            .events
            .iter()
            .position(|event| now.saturating_duration_since(event.time) >= self.buffering_time)
            .unwrap_or(self.events.len());

        self.events.drain(0..first_expired_index).collect()
    }

    fn get_errors(&mut self) -> Vec<NotifyError> {
        let mut v = Vec::new();
        std::mem::swap(&mut v, &mut self.errors);
        v
    }

    fn add_event(&mut self, event: NotifyEvent) {
        self.events.push(RawEvent::new(event, Instant::now()));
    }

    fn add_error(&mut self, error: NotifyError) {
        self.errors.push(error);
    }
}
