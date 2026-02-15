# NotifyKit

Notifykit is a cross-platform, high-performance filesystem event watcher for Python.

## Design Principles

- Modern, idiomatic Python codebase
- High performance and low latency
- Cross-platform support (Linux, macOS, Windows)
- Easy-to-use API for common use cases
- Extensible architecture for advanced use cases
- Robust error handling and logging
- Idiomatic, safe Rust for the PyO3 binding library

## Stack

- Python 3.10+ for the main codebase
- `uv` for Python package management
- `maturine` for building and publishing Python packages
- `pytest` for testing

## Testing

- `make lint-fix` - Run linters and fix issues
- `make lib-lint-fix` - Run linters and fix issues for the Rust library code
- `make test` - Run all tests