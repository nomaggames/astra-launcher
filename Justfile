# ASTRA Launcher - Justfile

# Default: show available commands
default:
    @just --list

# Install dependencies
install:
    npm install

# Run launcher in dev mode (loads .env for GitHub token)
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .env ]; then source .env; fi
    npm run dev

# Build launcher for production
build:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .env ]; then source .env; fi
    npm run build
