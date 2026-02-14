"""Tests for watch mechanics: recursive, non-recursive, multiple paths, unwatch, stop."""

import asyncio
import sys
from pathlib import Path

import pytest

from notifykit import CreateEvent, Notifier

from .conftest import DEBOUNCE_MS, SETTLE_DELAY, TICK_MS, collect_events, has_event


async def test_recursive_watching(watched_dir: Path, notifier: Notifier):
    """Events in nested subdirectories are reported."""
    nested = watched_dir / "a" / "b" / "c"
    nested.mkdir(parents=True)

    target = nested / "deep.txt"
    target.write_text("deep")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    assert has_event(events, CreateEvent, path=target)


@pytest.mark.skipif(sys.platform == "darwin", reason="FSEvents can leak subdirectory events on macOS")
async def test_non_recursive_watching(tmp_path: Path):
    """Subdirectory events NOT reported; root-level events ARE reported."""
    notifier = Notifier(debounce_ms=DEBOUNCE_MS, tick_ms=TICK_MS)

    try:
        await notifier.watch([tmp_path], recursive=False)
        await asyncio.sleep(0.05)

        subdir = tmp_path / "child"
        subdir.mkdir()

        await asyncio.sleep(SETTLE_DELAY)
        await collect_events(notifier)  # drain mkdir event

        nested_file = subdir / "nested.txt"
        nested_file.write_text("should not appear")

        root_file = tmp_path / "root.txt"
        root_file.write_text("should appear")

        await asyncio.sleep(SETTLE_DELAY)
        events = await collect_events(notifier)

        assert has_event(events, CreateEvent, path=root_file), f"Missing root event, got: {events}"
        assert not has_event(events, CreateEvent, path=nested_file), f"Got unexpected nested event: {events}"
    finally:
        notifier.stop()


async def test_unwatch(watched_dir: Path, notifier: Notifier):
    """After unwatch(), new events not reported."""
    # Verify watching works first
    probe = watched_dir / "probe.txt"
    probe.write_text("probe")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)
    assert has_event(events, CreateEvent, path=probe)

    # Unwatch
    await notifier.unwatch([watched_dir])
    await asyncio.sleep(0.05)

    # New events should not be reported
    ghost = watched_dir / "ghost.txt"
    ghost.write_text("ghost")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier, timeout=0.5)

    assert not has_event(events, CreateEvent, path=ghost), f"Got events after unwatch: {events}"


async def test_stop_ends_iteration(watched_dir: Path, notifier: Notifier):
    """stop() prevents new events from being produced."""
    # First establish the async iterator by collecting an event
    probe = watched_dir / "probe.txt"
    probe.write_text("probe")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)
    assert has_event(events, CreateEvent, path=probe), "Iterator not established"

    # Now stop
    notifier.stop()

    # Create a file — no new events should appear
    (watched_dir / "after_stop.txt").write_text("nope")
    await asyncio.sleep(SETTLE_DELAY)

    events = await collect_events(notifier, timeout=0.5)
    assert len(events) == 0, f"Expected no events after stop, got: {events}"


async def test_watch_nonexistent_path(notifier: Notifier):
    """FileNotFoundError raised when watching a nonexistent path."""
    with pytest.raises((FileNotFoundError, OSError)):
        await notifier.watch([Path("/nonexistent/path/that/does/not/exist")])


async def test_watch_multiple_paths(tmp_path: Path):
    """Events from two separate watched directories both reported."""
    dir_a = tmp_path / "dir_a"
    dir_b = tmp_path / "dir_b"
    dir_a.mkdir()
    dir_b.mkdir()

    notifier = Notifier(debounce_ms=DEBOUNCE_MS, tick_ms=TICK_MS)

    try:
        await notifier.watch([dir_a, dir_b], recursive=True)
        await asyncio.sleep(0.05)

        file_a = dir_a / "a.txt"
        file_b = dir_b / "b.txt"
        file_a.write_text("from A")
        file_b.write_text("from B")

        await asyncio.sleep(SETTLE_DELAY)
        events = await collect_events(notifier)

        assert has_event(events, CreateEvent, path=file_a), f"Missing event from dir_a, got: {events}"
        assert has_event(events, CreateEvent, path=file_b), f"Missing event from dir_b, got: {events}"
    finally:
        notifier.stop()
