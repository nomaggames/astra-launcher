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

# ============================================
# RELEASES
# ============================================
# Releases use v* tags (e.g., v0.1.0)
# GitHub Actions builds installers for all platforms

# Show current version from git tags
version:
    #!/usr/bin/env bash
    CURRENT=$(git tag -l 'v*' --sort=-v:refname | head -1)
    echo "Current version: ${CURRENT:-none}"

# Release - auto-increments patch version, commits all changes, tags, pushes
# Usage: just release         (auto patch bump: 0.1.0 → 0.1.1)
#        just release 0.2.0   (explicit version)
release *version:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT=$(git tag -l 'v*' --sort=-v:refname | head -1 | sed 's/^v//')
    if [ -z "$CURRENT" ]; then
        CURRENT="0.0.0"
    fi
    if [ -n "{{version}}" ]; then
        VERSION="{{version}}"
    else
        IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT"
        VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
    fi
    TAG="v$VERSION"
    echo "=== Releasing ASTRA Launcher $TAG ==="
    echo "Previous: v$CURRENT"
    echo ""
    git add -A
    if ! git diff --cached --quiet; then
        git commit -m "release: $TAG"
    else
        echo "No changes to commit"
    fi
    git tag "$TAG"
    git push origin main --tags
    echo ""
    echo "✓ Released $TAG"
    echo "  GitHub Actions will build installers for Windows/macOS/Linux"
