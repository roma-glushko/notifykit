.PHONY: help

SOURCE?=notifykit
TESTS?=tests

help:
	@echo "============="
	@echo "inotifykit 👀"
	@echo "============="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

lib-lint:  ## Lint the library codebase without changes (Rust)
	@cargo fmt --version
	@cargo fmt --all -- --check
	@cargo clippy --version
	@cargo clippy -- -D warnings

lib-lint-fix:  ## Lint the library codebase (Rust)
	@cargo fmt --version
	@cargo fmt --all
	@cargo clippy --version
	@cargo clippy -- -D warnings

lib-dev:  ## Build the library codebase as importable .so module
	@maturin develop

lint: ## Lint all Python source code without changes
	@pdm run ruff $(SOURCE)
	@pdm run ruff format $(SOURCE) --diff
	@pdm run mypy --pretty $(SOURCE)

lint-fix: ## Lint all source code
	@pdm run ruff --fix $(SOURCE)
	@pdm run ruff format $(SOURCE)
	@pdm run mypy --pretty $(SOURCE)
