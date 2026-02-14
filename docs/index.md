<p align="center">
  <img loading="lazy" src="./logo.png" width="400px" alt="notifykit">
</p>
<p align="center">
    <em>👀 A cross-platform filesystem watcher toolkit for Python</em>
</p>

**notifykit** is a set of components for building modern Python applications with a need for watching filesystem events efficiently.

## Installation

```bash
pip install notifykit
# or
uv add notifykit
```

notifykit is available for:

CPython 3.9+ on the following platforms:

- **Linux**: x86_64, aarch64, x86, armv7, s390x, ppc64le, musl-x86_64, musl-aarch64
- **MacOS**: x86_64 & arm64
- **Windows**: x64 & x86

PyPy 3.9+ on the following platforms:

- **Linux**: x86_64 & aarch64
- **MacOS**: x86_64

## Features

- Simple Modern Pythonic API (async)
- High Performance
- Cross-platform (Linux, MacOS, Windows)
- Built-in event filtering (`CommonFilter`, custom `EventFilter` subclasses)
- Easy to mock in tests
- Makes common cases easy and advanced cases possible
