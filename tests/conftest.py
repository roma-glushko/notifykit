import asyncio
from contextlib import asynccontextmanager

import async_timeout
from typing import AsyncContextManager, AsyncGenerator, Self

from notifykit import NotifierT, Event


class EventCollector:
    def __init__(self, ):
        self._events: list[Event] = []
        self._notifier_task: asyncio.Task | None = None

        self._waiters: set[tuple[int, asyncio.Event]] = set()

    @property
    def events(self) -> list[Event]:
        return self._events

    @asynccontextmanager
    async def collect(self, notifier: NotifierT) -> AsyncGenerator[Self, None]:
        if self._notifier_task is not None:
            raise RuntimeError("EventCollector is already running.")

        self._notifier_task = asyncio.create_task(self._collect_events(notifier))

        yield self

        notifier.stop()

        if self._notifier_task:
            self._notifier_task.cancel()
            try:
                await self._notifier_task
            except asyncio.CancelledError:
                pass
            self._notifier_task = None

    async def wait_for_events(self, items: int, timeout: float = 2) -> None:
        if items <= len(self._events):
            return

        waiter = asyncio.Event()
        waiter_id = (items, waiter)

        self._waiters.add(waiter_id)

        async with async_timeout.timeout(timeout):
            await waiter.wait()

        self._waiters.remove(waiter_id)

    async def _collect_events(self, notifier: NotifierT) -> None:
        async for event in notifier:
            self._events.extend(event)
            self._wakeup_waiters()

    def _wakeup_waiters(self) -> None:
        for items, event in self._waiters:
            if items <= len(self._events):
                event.set()
