#!/bin/bash

# Quick audit - just show long lines that need fixing
set -euo pipefail

# Determine files to scan
if [[ $# -gt 0 ]]; then
    # Custom paths provided
    for path in "$@"; do
        if [[ -f "$path" ]]; then
            awk 'length($0) > 100 {print FILENAME ":" NR ": " $0}' "$path"
        elif [[ -d "$path" ]]; then
            find "$path" -type f 2>/dev/null | while read -r file; do
                awk 'length($0) > 100 {print FILENAME ":" NR ": " $0}' "$file"
            done
        fi
    done
else
    # Default: uncommitted files
    git status --porcelain | sed 's/^[[:space:]]*[AMD?]//' | while read -r file; do
        if [[ -f "$file" ]]; then
            awk 'length($0) > 100 {print FILENAME ":" NR ": " $0}' "$file"
        fi
    done
fi
