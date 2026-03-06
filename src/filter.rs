use std::collections::HashSet;
use std::path::Path;

use regex::Regex;

use crate::events::EventType;

#[derive(Debug)]
pub(crate) struct EventFilter {
    ignore_dirs: HashSet<String>,
    ignore_patterns: Vec<Regex>,
    ignore_paths: Vec<String>,
}

impl EventFilter {
    pub fn new(
        ignore_dirs: Vec<String>,
        ignore_patterns: Vec<String>,
        ignore_paths: Vec<String>,
    ) -> Result<Self, regex::Error> {
        let compiled: Vec<Regex> = ignore_patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            ignore_dirs: ignore_dirs.into_iter().collect(),
            ignore_patterns: compiled,
            ignore_paths,
        })
    }

    /// Returns `true` if the event should be **dropped**.
    pub fn should_filter(&self, event: &EventType) -> bool {
        match event {
            EventType::Rename(e) => {
                // Both paths must be filtered for the rename event to be dropped
                self.should_filter_path(&e.old_path) && self.should_filter_path(&e.new_path)
            }
            _ => {
                let path = event.path().expect("non-rename event must have a path");
                self.should_filter_path(path)
            }
        }
    }

    fn should_filter_path(&self, path: &Path) -> bool {
        // Check if any path component matches ignore_dirs
        if !self.ignore_dirs.is_empty() {
            for component in path.components() {
                if let std::path::Component::Normal(name) = component {
                    if let Some(name_str) = name.to_str() {
                        if self.ignore_dirs.contains(name_str) {
                            return true;
                        }
                    }
                }
            }
        }

        // Check if filename matches any ignore pattern
        if !self.ignore_patterns.is_empty() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                for pattern in &self.ignore_patterns {
                    if pattern.is_match(file_name) {
                        return true;
                    }
                }
            }
        }

        // Check if path starts with any ignore_paths
        if !self.ignore_paths.is_empty() {
            let path_str = path.to_string_lossy();
            for ignore_path in &self.ignore_paths {
                if path_str.starts_with(ignore_path.as_str()) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::events::base::ObjectType;
    use crate::events::create::CreateEvent;
    use crate::events::rename::RenameEvent;

    fn make_filter() -> EventFilter {
        EventFilter::new(
            vec!["__pycache__".into(), ".git".into(), ".venv".into()],
            vec![r"\.py[cod]$".into(), "~$".into()],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn test_filter_by_dir() {
        let f = make_filter();
        assert!(f.should_filter_path(Path::new("/home/user/proj/__pycache__/mod.pyc")));
        assert!(f.should_filter_path(Path::new("/home/user/proj/.git/HEAD")));
        assert!(!f.should_filter_path(Path::new("/home/user/proj/main.py")));
    }

    #[test]
    fn test_filter_by_pattern() {
        let f = make_filter();
        assert!(f.should_filter_path(Path::new("/home/user/proj/mod.pyc")));
        assert!(f.should_filter_path(Path::new("/home/user/proj/logs.txt~")));
        assert!(!f.should_filter_path(Path::new("/home/user/proj/app.py")));
    }

    #[test]
    fn test_filter_by_ignore_path() {
        let f = EventFilter::new(vec![], vec![], vec!["/home/user/.cache".into()]).unwrap();

        assert!(f.should_filter_path(Path::new("/home/user/.cache/something")));
        assert!(!f.should_filter_path(Path::new("/home/user/proj/main.py")));
    }

    #[test]
    fn test_filter_event() {
        let f = make_filter();

        let event = EventType::Create(CreateEvent::new(
            PathBuf::from("/home/user/proj/__pycache__/mod.pyc"),
            ObjectType::File,
        ));
        assert!(f.should_filter(&event));

        let event = EventType::Create(CreateEvent::new(
            PathBuf::from("/home/user/proj/main.py"),
            ObjectType::File,
        ));
        assert!(!f.should_filter(&event));
    }

    #[test]
    fn test_filter_rename_both_filtered() {
        let f = make_filter();

        let event = EventType::Rename(RenameEvent::new(
            PathBuf::from("/home/user/proj/__pycache__/a"),
            PathBuf::from("/home/user/proj/.venv/b"),
        ));
        assert!(f.should_filter(&event));
    }

    #[test]
    fn test_filter_rename_one_not_filtered() {
        let f = make_filter();

        let event = EventType::Rename(RenameEvent::new(
            PathBuf::from("/home/user/proj/__pycache__/a"),
            PathBuf::from("/home/user/proj/real_file"),
        ));
        assert!(!f.should_filter(&event));
    }

    #[test]
    fn test_empty_filter() {
        let f = EventFilter::new(vec![], vec![], vec![]).unwrap();
        assert!(!f.should_filter_path(Path::new("/any/path/file.txt")));
    }
}
