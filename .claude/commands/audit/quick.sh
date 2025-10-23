#!/bin/bash

# Quick audit - just show long lines that need fixing
set -euo pipefail

git status --porcelain | grep -E '^.*\.rs$' | sed 's/^[[:space:]]*[AMD?]//' | grep '\.rs$' | while read -r file; do
    if [[ -f "$file" ]]; then
        awk 'length($0) > 100 {print FILENAME ":" NR ": " $0}' "$file"
    fi
done
