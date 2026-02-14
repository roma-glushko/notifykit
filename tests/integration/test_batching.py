"""Tests for event batching and debouncing behaviour."""

import asyncio
from pathlib import Path

from notifykit import CreateEvent, ModifyDataEvent, ModifyMetadataEvent, Notifier

from .conftest import SETTLE_DELAY, collect_events, find_events, has_event


async def test_multiple_events_batched(watched_dir: Path, notifier: Notifier):
    """Multiple rapid file creates all appear in collected events."""
    files = [watched_dir / f"file_{i}.txt" for i in range(5)]
    for f in files:
        f.write_text(f"content {f.name}")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    for f in files:
        assert has_event(events, CreateEvent, path=f), f"Missing CreateEvent for {f}"


async def test_rapid_changes_batched(watched_dir: Path, notifier: Notifier):
    """Multiple rapid writes are coalesced by the debouncer."""
    target = watched_dir / "rapid.txt"
    target.write_text("v0")

    await asyncio.sleep(SETTLE_DELAY)
    await collect_events(notifier)  # drain

    # Perform rapid writes
    for i in range(10):
        target.write_text(f"v{i + 1}")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    modify_events = find_events(events, ModifyDataEvent) + find_events(events, ModifyMetadataEvent)
    # We wrote 10 times but debouncing should coalesce — we expect fewer than 10 events
    assert len(modify_events) > 0, "Expected at least some modify events"
    assert len(modify_events) < 10, f"Expected debouncing to coalesce, got {len(modify_events)} events"
