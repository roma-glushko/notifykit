.PHONY: help

SOURCE?=notifykit
TESTS?=tests

help:
	@echo "============="
	@echo "notifykit 👀"
	@echo "============="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

tools: ## Install development tools
	@echo "Installing development tools..."
	@uv tool install maturin

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

build: ## Build the library
	@uv build

build-linux: ## Build the library for Linux (x86_64, abi3 wheel)
	@docker run --rm \
       -v "$(PWD)":/io \
       ghcr.io/pyo3/maturin:latest \
       build --release --out dist

lib-dev: tools  ## Build the library codebase as importable .so module
	@uvx maturin develop

lib-release: ## Build an optimized version of the .so module
	@uvx maturin build -r

test: ## Run all tests
	@uv run pytest $(TESTS)

test-v: ## Run all tests (verbose)
	@uv run pytest $(TESTS) -v

lint: ## Lint all Python source code without changes
	@uv run ruff check $(SOURCE)
	@uv run ruff format $(SOURCE) --diff
	@uv run mypy --pretty $(SOURCE)

lint-fix: ## Lint all source code
	@uv run ruff check --fix $(SOURCE)
	@uv run ruff format $(SOURCE)
	@uv run mypy --pretty $(SOURCE)

docs-serve: ## Run documentation locally
	@pdm run mkdocs serve -a localhost:7756

docs-build: ## Make a publishable version of documentation
	@pdm run mkdocs build

.PHONY: clean
clean:  # Clean all cache dirs
	@rm -rf `find . -name __pycache__`
	@rm -f `find . -type f -name '*.py[co]' `
	@rm -f `find . -type f -name '*.so' `
	@rm -f `find . -type f -name '*~' `
	@rm -f `find . -type f -name '.*~' `
	@rm -f tests/__init__.py
	@rm -rf .cache
	@rm -rf htmlcov
	@rm -rf .pytest_cache
	@rm -rf .mypy_cache
	@rm -rf *.egg-info
	@rm -f .coverage
	@rm -f .coverage.*
	@rm -rf build
	@rm -rf dist
	@rm -rf sdist
	@rm -rf site
	@rm -rf .ruff_cache
	@rm -rf target
