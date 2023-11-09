.PHONY: help

help:
	@echo "============="
	@echo "inotifykit 👀"
	@echo "============="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

lib-lint:  ## Lint the library codebase (Rust)
	@cargo fmt

lib-dev:  ## Build the library codebase as importable .so module
	@maturin develop
