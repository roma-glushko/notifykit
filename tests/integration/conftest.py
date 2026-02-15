"""Shared fixtures and helpers for integration tests."""

import asyncio
from pathlib import Path
from typing import List

import pytest

from notifykit import Event, Notifier

# Fast timing for tests
DEBOUNCE_MS = 50
TICK_MS = 25

# Sleep after FS ops to let events propagate past the debounce window
SETTLE_DELAY = 0.15

# Safety timeout waiting for the first batch; shorter for subsequent batches
COLLECT_TIMEOUT = 3.0
SUBSEQUENT_TIMEOUT = 0.3


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


async def collect_events(
    notifier: Notifier,
    timeout: float = COLLECT_TIMEOUT,
    subsequent_timeout: float = SUBSEQUENT_TIMEOUT,
    max_batches: int = 50,
) -> List[Event]:
    """Collect event batches from *notifier*, flattened into a single list.

    Uses a longer *timeout* for the first batch (the watcher may need time to
    deliver), then a shorter *subsequent_timeout* for follow-ups.
    """
    all_events: List[Event] = []

    for i in range(max_batches):
        wait = timeout if i == 0 else subsequent_timeout
        try:
            batch = await asyncio.wait_for(notifier.__anext__(), timeout=wait)
            all_events.extend(batch)
        except (asyncio.TimeoutError, StopAsyncIteration):
            break

    return all_events


def has_event(events: List[Event], event_type: type, **attrs: object) -> bool:
    """Return True if *events* contains an event of *event_type* whose
    attributes match all *attrs* (compared via ``str()`` for paths)."""
    for ev in events:
        if not isinstance(ev, event_type):
            continue
        if all(str(getattr(ev, k, None)) == str(v) for k, v in attrs.items()):
            return True
    return False


def find_events(events: List[Event], event_type: type) -> List[Event]:
    """Return all events in *events* that are instances of *event_type*."""
    return [ev for ev in events if isinstance(ev, event_type)]


# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------


@pytest.fixture
def notifier():
    n = Notifier(debounce_ms=DEBOUNCE_MS, tick_ms=TICK_MS)
    yield n
    n.stop()


@pytest.fixture
async def watched_dir(tmp_path: Path, notifier: Notifier):
    await notifier.watch([tmp_path], recursive=True)
    await asyncio.sleep(0.05)  # give watcher time to initialise
    yield tmp_path
