import re
from pathlib import Path
from typing import Sequence, Union

from notifykit._notifykit_lib import RenameEvent

from notifykit._typing import Event


class EventFilter:
    """
    A base class to define rules to filter filesystem events
    TODO: Move filtering to the Rust library
    """

    __slots__ = "_ignore_dirs", "_ignore_object_regexes", "_ignore_paths"

    ignore_dirs: Sequence[str] = ()
    """Full names of directories to ignore like `.git`."""

    ignore_object_patterns: Sequence[str] = ()
    """
    Patterns of files or directories to ignore, these are compiled into regexes.
    """

    ignore_paths: Sequence[Union[str, Path]] = ()
    """
    Full paths to ignore, e.g. `/home/users/.cache` or `C:\\Users\\user\\.cache`.
    """

    def __init__(self) -> None:
        self._ignore_dirs = set(self.ignore_dirs)
        self._ignore_object_regexes = tuple(re.compile(r) for r in self.ignore_object_patterns)
        self._ignore_paths = tuple(map(str, self.ignore_paths))

    def __call__(self, event: Event) -> bool:
        """
        Check if event should be filtered (True) or kept in place (False)
        """
        if isinstance(event, RenameEvent):
            return self._should_be_filtered(Path(event.old_path)) and self._should_be_filtered(Path(event.new_path))

        return self._should_be_filtered(Path(event.path))

    def _should_be_filtered(self, path: Path) -> bool:
        if any(p in self._ignore_dirs for p in path.parts):
            return True

        object_name = path.name

        if any(r.search(object_name) for r in self._ignore_object_regexes):
            return True

        if self._ignore_paths:
            for ignore_path in self._ignore_paths:
                if path.is_relative_to(ignore_path):
                    return True

        return False

    def __repr__(self) -> str:
        args = ", ".join(f"{k}={getattr(self, k, None)!r}" for k in self.__slots__)

        return f"{self.__class__.__name__}({args})"


class CommonFilter(EventFilter):
    """
    Filter commonly ignored files and directories
    """

    ignore_dirs: Sequence[str] = (
        "__pycache__",
        ".git",
        ".hg",
        ".svn",
        ".tox",
        ".venv",
        "site-packages",
        ".idea",
        "node_modules",
        ".mypy_cache",
        ".ruff_cache",
        ".pytest_cache",
        ".hypothesis",
    )

    ignore_object_patterns: Sequence[str] = (
        r"\.py[cod]$",
        r"\.___jb_...___$",
        r"\.sw.$",
        "~$",
        r"^\.\#",
        r"^\.DS_Store$",
        r"^flycheck_",
    )
