<p align="center">
  <img loading="lazy" src="https://raw.githubusercontent.com/roma-glushko/notifykit/main/imgs/logo.png" width="400px" alt="notifykit">
</p>
<p align="center">
    <em>👀 A cross-platform filesystem watcher toolkit for Python</em>
</p>

<a href="https://pypi.org/project/notifykit/"><img src="https://img.shields.io/pypi/v/notifykit" alt="PyPI - Version"></a>
<a href="https://pypi.org/project/notifykit/"><img src="https://img.shields.io/pypi/dm/notifykit" alt="PyPI - Downloads"></a>
![GitHub License](https://img.shields.io/github/license/roma-glushko/notifykit)

**notifykit** is a set of components for building modern Python applications with a need for watching filesystem events efficiently.

> [!Note]
> `notifykit` has been running successfully in production for 2+ years.

## Installation

```bash
pip install notifykit
# or
uv add notifykit
```

notifykit is available for:

CPython 3.10+ on the following platforms:

- **Linux**: x86_64, aarch64, x86, armv7, s390x, ppc64le, musl-x86_64, musl-aarch64
- **MacOS**: x86_64 & arm64
- **Windows**: x64 & x86

PyPy 3.10+ on the following platforms:

- **Linux**: x86_64 & aarch64
- **MacOS**: x86_64

## Usage

```python
import asyncio
from pathlib import Path

from notifykit import Notifier, CommonFilter


async def watch(watched_dir: Path) -> None:
    notifier = Notifier(
        debounce_ms=200,
        filter=CommonFilter(),
    )
    await notifier.watch([watched_dir])

    async for events in notifier:
        # process your events
        print(events)


if __name__ == "__main__":
    watched_dir = Path("./watched_dir")
    watched_dir.mkdir(exist_ok=True)

    asyncio.run(watch(watched_dir))
```

## Features

- Simple Modern Pythonic API (async)
- High Performance
- Cross-platform (Linux, MacOS, Windows)
- Built-in event filtering (`CommonFilter`, custom `EventFilter` subclasses)
- Easy to mock in tests
- Makes common cases easy and advanced cases possible

## Sources of Inspiration

- https://github.com/seb-m/pyinotify/issues
- https://github.com/absperf/asyncinotify/
- https://docs.rs/notify/latest/notify/
- https://github.com/samuelcolvin/watchfiles
- https://github.com/pantsbuild/pants/tree/612e891e90432e994327b6ddaf57502366a714c0/src/rust/engine
- https://github.com/pola-rs/polars/blob/d0c8de592b71d4b934b1598926536f03e10007bd/py-polars/src/file.rs#L206
- https://github.com/TheoBabilon/async-tail/
