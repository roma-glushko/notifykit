"""Tests for directory-level events: create and delete."""

import asyncio
from pathlib import Path

from notifykit import CreateEvent, DeleteEvent, Notifier

from .conftest import SETTLE_DELAY, collect_events, has_event


async def test_dir_creation(watched_dir: Path, notifier: Notifier):
    """CreateEvent emitted for a new directory."""
    subdir = watched_dir / "newdir"
    subdir.mkdir()

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    assert has_event(events, CreateEvent, path=subdir)


async def test_dir_deletion(watched_dir: Path, notifier: Notifier):
    """DeleteEvent emitted for a removed directory."""
    subdir = watched_dir / "gonedir"
    subdir.mkdir()

    await asyncio.sleep(SETTLE_DELAY)
    await collect_events(notifier)  # drain

    subdir.rmdir()

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    assert has_event(events, DeleteEvent, path=subdir)
