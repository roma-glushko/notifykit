.PHONY: help

SOURCE?=notifykit
TESTS?=tests

help:
	@echo "============="
	@echo "inotifykit 👀"
	@echo "============="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

lib-lint:  ## Lint the library codebase (Rust)
	@cargo fmt --version
	@cargo fmt --all -- --check
	@cargo clippy --version
	@cargo clippy -- -D warnings

lib-dev:  ## Build the library codebase as importable .so module
	@maturin develop

lint: ## Lint all source code
	@ruff --fix $(SOURCE)
	@mypy --pretty $(SOURCE)

