<p align="center">
  <img loading="lazy" src="https://raw.githubusercontent.com/roma-glushko/notifykit/main/imgs/logo.png" width="400px" alt="notifykit">
</p>
<p align="center">
    <em>👀 A cross-platform filesystem watcher toolkit for Python</em>
</p>

**notifykit** is a set of components for building modern Python applications with a need for watching filesystem events efficiently.

> [!Warning]
> notifykit is under active development right now

## Installation

```bash
pip install notifykit
# or
poetry add notifykit
# or 
pdm add notifykit
```

notifykit is available for:

CPython 3.8-3.12 on the following platforms:

- **Linux**: x86_64, aarch64, x86, armv7, s390x, ppc64le, musl-x86_64, musl-aarch64
- **MacOS**: x86_64 & arm64
- **Windows**: x64 & x86

PyPY 3.8-3.10 on the following platforms:

- **Linux**: x86_64 & aarch64
- **MacOS**: x86_64

## Usage

```python
import asyncio
import os
from pathlib import Path

from notifykit import Notifier


async def watch(watched_dir: Path) -> None:
    with Notifier(debounce_ms=200, debug=True) as notifier:
        notifier.watch([watched_dir])

        async for event in notifier:
            # process your events
            print(event)


if __name__ == "__main__":
    watched_dir = Path("./watched_dir")
    os.makedirs(watched_dir, exist_ok=True)

    asyncio.run(watch(watched_dir))
```

## Features

- Simple Modern Pythonic API, both sync and async
- High Performance
- Cross-platform (support not only Linux, but also MacOS)
- Easy to mock in tests
- Makes common cases easy and advance cases possible

## Sources of Inspiration

- https://github.com/seb-m/pyinotify/issues
- https://github.com/absperf/asyncinotify/
- https://docs.rs/notify/latest/notify/
- https://github.com/samuelcolvin/watchfiles
- https://github.com/pantsbuild/pants/tree/612e891e90432e994327b6ddaf57502366a714c0/src/rust/engine
- https://github.com/pola-rs/polars/blob/d0c8de592b71d4b934b1598926536f03e10007bd/py-polars/src/file.rs#L206
- https://github.com/TheoBabilon/async-tail/
