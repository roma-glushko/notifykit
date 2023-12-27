use std::collections::{HashMap, VecDeque};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::time::Duration;

#[cfg(test)]
use mock_instant::Instant;

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

#[derive(Debug)]
pub struct EventProcessor<T> {
    events_by_file: HashMap<PathBuf, FileEventQueue>,
    file_cache: T,
    rename_event: Option<(RawEvent, Option<FileId>)>,
    rescan_event: Option<RawEvent>,
    errors: Vec<NotifyError>,
    buffering_time: Duration,
}

impl<T: FileIdCache> EventProcessor<T> {
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

    pub fn get_events(&mut self) -> Vec<RawEvent> {
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
    pub fn get_errors(&mut self) -> Vec<NotifyError> {
        let mut v = Vec::new();
        std::mem::swap(&mut v, &mut self.errors);
        v
    }

    /// Add an error entry to re-send later on
    pub fn add_error(&mut self, error: NotifyError) {
        self.errors.push(error);
    }

    /// Add new event to debouncer cache
    pub fn add_event(&mut self, event: NotifyEvent) {
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

#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path};

    use rstest::rstest;
    use super::*;

    use mock_instant::MockClock;
    use crate::file_cache::FileCacheMock;

    use std::collections::HashMap;

    use serde::Deserialize;

    use notify::{
        event::{
            AccessKind, AccessMode, CreateKind, DataChange, Flag, MetadataKind, ModifyKind, RemoveKind,
            RenameMode,
        },
        EventKind,
    };

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub(crate) struct Error {
        /// The error kind is parsed by `into_notify_error`
        pub kind: String,

        /// The error paths
        #[serde(default)]
        pub paths: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub(crate) struct TestEvent {
        /// The timestamp the event occurred
        #[serde(default)]
        pub time: u64,

        /// The event kind is parsed by `into_notify_event`
        pub kind: String,

        /// The event paths
        #[serde(default)]
        pub paths: Vec<String>,

        /// The event flags
        #[serde(default)]
        pub flags: Vec<String>,

        /// The event tracker
        pub tracker: Option<usize>,

        /// The event info
        pub info: Option<String>,

        /// The file id for the file associated with the event
        ///
        /// Only used for the rename event.
        pub file_id: Option<u64>,
    }

    impl TestEvent {
        #[rustfmt::skip]
        pub fn into_raw_event(self, time: Instant, path: Option<&str>) -> RawEvent {
            let kind = match &*self.kind {
                "any" => EventKind::Any,
                "other" => EventKind::Other,
                "access-any" => EventKind::Access(AccessKind::Any),
                "access-read" => EventKind::Access(AccessKind::Read),
                "access-open-any" => EventKind::Access(AccessKind::Open(AccessMode::Any)),
                "access-open-execute" => EventKind::Access(AccessKind::Open(AccessMode::Execute)),
                "access-open-read" => EventKind::Access(AccessKind::Open(AccessMode::Read)),
                "access-open-write" => EventKind::Access(AccessKind::Open(AccessMode::Write)),
                "access-open-other" => EventKind::Access(AccessKind::Open(AccessMode::Other)),
                "access-close-any" => EventKind::Access(AccessKind::Close(AccessMode::Any)),
                "access-close-execute" => EventKind::Access(AccessKind::Close(AccessMode::Execute)),
                "access-close-read" => EventKind::Access(AccessKind::Close(AccessMode::Read)),
                "access-close-write" => EventKind::Access(AccessKind::Close(AccessMode::Write)),
                "access-close-other" => EventKind::Access(AccessKind::Close(AccessMode::Other)),
                "access-other" => EventKind::Access(AccessKind::Other),
                "create-any" => EventKind::Create(CreateKind::Any),
                "create-file" => EventKind::Create(CreateKind::File),
                "create-folder" => EventKind::Create(CreateKind::Folder),
                "create-other" => EventKind::Create(CreateKind::Other),
                "modify-any" => EventKind::Modify(ModifyKind::Any),
                "modify-other" => EventKind::Modify(ModifyKind::Other),
                "modify-data-any" => EventKind::Modify(ModifyKind::Data(DataChange::Any)),
                "modify-data-size" => EventKind::Modify(ModifyKind::Data(DataChange::Size)),
                "modify-data-content" => EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                "modify-data-other" => EventKind::Modify(ModifyKind::Data(DataChange::Other)),
                "modify-metadata-any" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
                "modify-metadata-access-time" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::AccessTime)),
                "modify-metadata-write-time" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::WriteTime)),
                "modify-metadata-permissions" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::Permissions)),
                "modify-metadata-ownership" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::Ownership)),
                "modify-metadata-extended" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::Extended)),
                "modify-metadata-other" => EventKind::Modify(ModifyKind::Metadata(MetadataKind::Other)),
                "rename-any" => EventKind::Modify(ModifyKind::Name(RenameMode::Any)),
                "rename-from" => EventKind::Modify(ModifyKind::Name(RenameMode::From)),
                "rename-to" => EventKind::Modify(ModifyKind::Name(RenameMode::To)),
                "rename-both" => EventKind::Modify(ModifyKind::Name(RenameMode::Both)),
                "rename-other" => EventKind::Modify(ModifyKind::Name(RenameMode::Other)),
                "remove-any" => EventKind::Remove(RemoveKind::Any),
                "remove-file" => EventKind::Remove(RemoveKind::File),
                "remove-folder" => EventKind::Remove(RemoveKind::Folder),
                "remove-other" => EventKind::Remove(RemoveKind::Other),
                _ => panic!("unknown event type `{}`", self.kind),
            };
            let mut event = notify::Event::new(kind);

            for p in self.paths {
                event = event.add_path(if p == "*" {
                    PathBuf::from(path.expect("cannot replace `*`"))
                } else {
                    PathBuf::from(p)
                });

                if let Some(tracker) = self.tracker {
                    event = event.set_tracker(tracker);
                }

                if let Some(info) = &self.info {
                    event = event.set_info(info.as_str());
                }
            }

            for f in self.flags {
                let flag = match &*f {
                    "rescan" => Flag::Rescan,
                    _ => panic!("unknown event flag `{f}`"),
                };

                event = event.set_flag(flag);
            }

            RawEvent { event, time: time + Duration::from_millis(self.time) }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    pub(crate) struct TestCase {
        /// A map of file ids, used instead of accessing the file system
        #[serde(default)]
        pub file_system: HashMap<String, u64>,

        /// Cached file ids
        #[serde(default)]
        pub cache: HashMap<String, u64>,

        /// Incoming events that are added during the test
        #[serde(default)]
        pub events: Vec<TestEvent>,

        /// Incoming errors that are added during the test
        #[serde(default)]
        pub errors: Vec<Error>,

        /// Events expected to get after processing
        #[serde(default)]
        pub processed_events: Vec<TestEvent>,

        #[serde(default)]
        pub processed_errors: Vec<Error>,
    }

    #[rstest]
    fn test_processor_output_events(
        #[values("atomic_file_update")]
        test_case_name: &str
    ) {
        println!("CWD: {:?}", env::current_dir().unwrap());

        let content = fs::read_to_string(Path::new(&format!("./testcases/processor/{test_case_name}.hjson"))).unwrap();
        let mut test_case = deser_hjson::from_str::<TestCase>(&content).unwrap();

        MockClock::set_time(Duration::default());

        let time = Instant::now();

        let cache = test_case
            .cache
            .into_iter()
            .map(|(path, id)| {
                let path = PathBuf::from(path);
                let id = FileId::new_inode(id, id);
                (path, id)
            })
            .collect::<HashMap<_, _>>();

        let file_system = test_case
            .file_system
            .into_iter()
            .map(|(path, id)| {
                let path = PathBuf::from(path);
                let id = FileId::new_inode(id, id);
                (path, id)
            })
            .collect::<HashMap<_, _>>();

        let file_cache = FileCacheMock::new(cache, file_system);
        let mut processor = EventProcessor::new(file_cache, Duration::from_millis(20));

        for event in test_case.events {
            let event = event.into_raw_event(time, None);
            MockClock::set_time(event.time - time);

            processor.add_event(event.event);
        }

        // let expected_errors = std::mem::take(&mut test_case.expected.errors);
        let expected_events = std::mem::take(&mut test_case.processed_events);

        // for error in test_case.errors {
        //     let e = error.into_notify_error();
        //     state.add_error(e);
        // }
    }
}