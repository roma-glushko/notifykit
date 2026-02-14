"""Tests for basic file-level events: create, modify, delete, rename."""

import asyncio
from pathlib import Path

from notifykit import (
    CreateEvent,
    DeleteEvent,
    ModifyDataEvent,
    ModifyMetadataEvent,
    Notifier,
    RenameEvent,
)

from .conftest import SETTLE_DELAY, collect_events, find_events, has_event


async def test_file_creation(watched_dir: Path, notifier: Notifier):
    """CreateEvent emitted when a file is created."""
    target = watched_dir / "hello.txt"
    target.write_text("hello")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    assert has_event(events, CreateEvent, path=target)


async def test_file_modification(watched_dir: Path, notifier: Notifier):
    """ModifyDataEvent or ModifyMetadataEvent emitted on write."""
    target = watched_dir / "data.txt"
    target.write_text("initial")

    await asyncio.sleep(SETTLE_DELAY)
    # drain creation events
    await collect_events(notifier)

    target.write_text("modified")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    modify_events = find_events(events, ModifyDataEvent) + find_events(events, ModifyMetadataEvent)
    assert len(modify_events) > 0, f"Expected modify events, got: {events}"


async def test_file_deletion(watched_dir: Path, notifier: Notifier):
    """DeleteEvent emitted when a file is removed."""
    target = watched_dir / "to_delete.txt"
    target.write_text("bye")

    await asyncio.sleep(SETTLE_DELAY)
    await collect_events(notifier)  # drain

    target.unlink()

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    assert has_event(events, DeleteEvent, path=target)


async def test_file_rename(watched_dir: Path, notifier: Notifier):
    """RenameEvent OR Delete+Create pair emitted on rename (platform-dependent)."""
    src = watched_dir / "old_name.txt"
    dst = watched_dir / "new_name.txt"
    src.write_text("rename me")

    await asyncio.sleep(SETTLE_DELAY)
    await collect_events(notifier)  # drain

    src.rename(dst)

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    got_rename = has_event(events, RenameEvent)
    got_delete_create = has_event(events, DeleteEvent, path=src) and has_event(events, CreateEvent, path=dst)

    assert got_rename or got_delete_create, f"Expected rename or delete+create, got: {events}"
