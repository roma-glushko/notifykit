import os
from pathlib import Path

from notifykit import Notifier


def watch(watched_dir: Path) -> None:
    notifier = Notifier(debounce_ms=200, debug=True)

    notifier.watch([watched_dir])

    for events in notifier:
        print(events)


if __name__ == "__main__":
    watched_dir = Path("./watched_dir")
    os.makedirs(watched_dir, exist_ok=True)

    watch(watched_dir)
