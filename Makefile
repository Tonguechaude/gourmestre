# Yes sorry you need to install tailwind f*** NodeJS
TAILWIND = /usr/local/bin/tailwindcss

CSS_WORKDIR = frontend
OUTPUT = frontend/static/style.css
INPUT = frontend/tailwind/tailwind.css
CONFIG = frontend/tailwind.config.js

# Command to build
build-css:
	@echo "🔧 Compilation Tailwind CSS..."
	cd frontend && \
	npx @tailwindcss/cli -i ./tailwind/input.css -o ./static/style.css --minify
	@echo "CSS compilé dans frontend/static/style.css"

# Command to watch changes in CSS
watch-css:
	@echo "Watching Tailwind CSS for changes..."
	cd frontend && \
	npx @tailwindcss/cli -i ./tailwind/input.css -o ./static/style.css --watch

# Default goal and help
.DEFAULT_GOAL := help
help:
	@grep -E '(^[a-zA-Z_-]+:.*?##.*$$)|(^##)' $(MAKEFILE_LIST) | sed -e 's/^Makefile:\(.*\)/\1/' | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[32m%-30s\033[0m %s\n", $$1, $$2}' | sed -e 's/\[32m##/[33m/'

.PHONY: help

