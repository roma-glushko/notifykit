import asyncio
import os
import time
from pathlib import Path

from notifykit import Notifier

notifier = Notifier(debounce_ms=200, debug=True)

async def hangout():
    c = 0

    while True:
        if c == 10:
            notifier.stop()
            break

        print(c)
        c += 1
        s = time.monotonic_ns()
        await asyncio.sleep(0.1)
        print(f"took {(time.monotonic_ns() - s) / 1e6:.2f}ms")


async def watch(watched_dir: Path) -> None:
    await notifier.watch([watched_dir])

    async for events in notifier:
        # process your events
        print(events)

async def main():
    watched_dir = Path("./watched_dir").absolute()
    os.makedirs(watched_dir, exist_ok=True)

    # asyncio.create_task(hangout())
    await watch(watched_dir)


if __name__ == "__main__":
    watched_dir = Path("./watched_dir")
    os.makedirs(watched_dir, exist_ok=True)

    asyncio.run(main())
