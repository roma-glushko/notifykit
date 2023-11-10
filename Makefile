.PHONY: help

help:
	@echo "============="
	@echo "notifykit 👀"
	@echo "============="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

lib-lint:  ## Lint the library codebase (Rust)
	@cargo fmt

lib-dev:  ## Build the library codebase as importable .so module
	@maturin develop

lib-release: ## Build an optimized version of the .so module
	@maturin build -r

docs-serve: ## Run documentation locally
	@mkdocs serve -a localhost:7756

docs-build: ## Make a publishable version of documentation
	@mkdocs build

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
