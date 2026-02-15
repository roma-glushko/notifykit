"""Tests for event batching and debouncing behaviour."""

import asyncio
from pathlib import Path
from typing import List

from notifykit import CreateEvent, Event, ModifyDataEvent, Notifier

from .conftest import COLLECT_TIMEOUT, SETTLE_DELAY, SUBSEQUENT_TIMEOUT, collect_events, find_events, has_event


async def test_multiple_events_batched(watched_dir: Path, notifier: Notifier):
    """Multiple rapid file creates all appear in collected events."""
    files = [watched_dir / f"file_{i}.txt" for i in range(5)]
    for f in files:
        f.write_text(f"content {f.name}")

    await asyncio.sleep(SETTLE_DELAY)
    events = await collect_events(notifier)

    for f in files:
        assert has_event(events, CreateEvent, path=f), f"Missing CreateEvent for {f}"


async def _collect_batches(
    notifier: Notifier,
    timeout: float = COLLECT_TIMEOUT,
    subsequent_timeout: float = SUBSEQUENT_TIMEOUT,
    max_batches: int = 50,
) -> List[List[Event]]:
    """Collect raw event batches without flattening."""
    batches: List[List[Event]] = []

    for i in range(max_batches):
        wait = timeout if i == 0 else subsequent_timeout
        try:
            batch = await asyncio.wait_for(notifier.__anext__(), timeout=wait)
            batches.append(batch)
        except (asyncio.TimeoutError, StopAsyncIteration):
            break

    return batches


async def test_rapid_changes_batched(watched_dir: Path, notifier: Notifier):
    """Rapid writes are delivered in fewer batches than total writes."""
    target = watched_dir / "rapid.txt"
    target.write_text("v0")

    await asyncio.sleep(SETTLE_DELAY)
    await collect_events(notifier)  # drain

    # Perform rapid writes — these complete in microseconds
    num_writes = 10
    for i in range(num_writes):
        target.write_text(f"v{i + 1}")

    await asyncio.sleep(SETTLE_DELAY)
    batches = await _collect_batches(notifier)

    all_modify = [e for batch in batches for e in find_events(batch, ModifyDataEvent)]

    assert len(all_modify) > 0, "Expected at least some modify events"
    # The debouncer groups events by time window — 10 rapid writes should arrive
    # in fewer batches than individual writes, proving batching works
    assert len(batches) < num_writes, (
        f"Expected fewer batches than writes, got {len(batches)} batches for {num_writes} writes"
    )
