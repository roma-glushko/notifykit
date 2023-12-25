import asyncio
import os
from pathlib import Path

from notifykit import Notifier


async def watch(watched_dir: Path) -> None:
    notifier = Notifier(debounce_ms=200, debug=True)

    notifier.watch([watched_dir])

    async for events in notifier:
        # process your events
        print(events)


if __name__ == "__main__":
    watched_dir = Path("./watched_dir")
    os.makedirs(watched_dir, exist_ok=True)

    asyncio.run(watch(watched_dir))
