"""Tests for event filtering: CommonFilter and custom EventFilter subclasses."""

import asyncio
from pathlib import Path

from notifykit import CommonFilter, CreateEvent, EventFilter, Notifier

from .conftest import DEBOUNCE_MS, SETTLE_DELAY, TICK_MS, collect_events, has_event


async def test_common_filter(tmp_path: Path):
    """.git/, __pycache__/, .pyc events filtered; normal files pass through."""
    notifier = Notifier(debounce_ms=DEBOUNCE_MS, tick_ms=TICK_MS, filter=CommonFilter())

    try:
        await notifier.watch([tmp_path], recursive=True)
        await asyncio.sleep(0.05)

        # Create filtered paths
        git_dir = tmp_path / ".git"
        git_dir.mkdir()
        (git_dir / "HEAD").write_text("ref: refs/heads/main")

        pycache_dir = tmp_path / "__pycache__"
        pycache_dir.mkdir()
        (pycache_dir / "mod.cpython-312.pyc").write_bytes(b"\x00")

        pyc_file = tmp_path / "something.pyc"
        pyc_file.write_bytes(b"\x00")

        # Create a normal file that should pass through
        normal = tmp_path / "app.py"
        normal.write_text("print('hello')")

        await asyncio.sleep(SETTLE_DELAY)
        events = await collect_events(notifier)

        assert has_event(events, CreateEvent, path=normal), f"Normal file event missing, got: {events}"
        assert not has_event(events, CreateEvent, path=git_dir / "HEAD"), f"Got .git event: {events}"
        assert not has_event(
            events, CreateEvent, path=pycache_dir / "mod.cpython-312.pyc"
        ), f"Got __pycache__ event: {events}"
        assert not has_event(events, CreateEvent, path=pyc_file), f"Got .pyc event: {events}"
    finally:
        notifier.stop()


async def test_custom_filter(tmp_path: Path):
    """Subclassed EventFilter with custom ignore_dirs works."""

    class IgnoreLogs(EventFilter):
        ignore_dirs = ("logs",)

    notifier = Notifier(debounce_ms=DEBOUNCE_MS, tick_ms=TICK_MS, filter=IgnoreLogs())

    try:
        await notifier.watch([tmp_path], recursive=True)
        await asyncio.sleep(0.05)

        logs_dir = tmp_path / "logs"
        logs_dir.mkdir()
        (logs_dir / "app.log").write_text("log entry")

        normal = tmp_path / "main.py"
        normal.write_text("print('hi')")

        await asyncio.sleep(SETTLE_DELAY)
        events = await collect_events(notifier)

        assert has_event(events, CreateEvent, path=normal), f"Normal file event missing, got: {events}"
        assert not has_event(events, CreateEvent, path=logs_dir / "app.log"), f"Got logs/ event: {events}"
    finally:
        notifier.stop()
