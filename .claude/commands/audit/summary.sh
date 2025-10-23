#!/bin/bash

# Quick summary of audit status
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Determine files to scan
if [[ $# -gt 0 ]]; then
    # Custom paths provided
    FILES_TO_SCAN=""
    for path in "$@"; do
        if [[ -f "$path" ]]; then
            FILES_TO_SCAN="$FILES_TO_SCAN$path\n"
        elif [[ -d "$path" ]]; then
            FOUND_FILES=$(find "$path" -type f 2>/dev/null || true)
            if [[ -n "$FOUND_FILES" ]]; then
                FILES_TO_SCAN="$FILES_TO_SCAN$FOUND_FILES\n"
            fi
        fi
    done
    FILES_TO_SCAN=$(echo -e "$FILES_TO_SCAN" | grep -v '^$' || true)
    SCAN_TYPE="custom paths"
else
    # Default: uncommitted files
    if ! command -v git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}Not in git repository and no custom paths provided${NC}"
        exit 1
    fi
    FILES_TO_SCAN=$(git status --porcelain | sed 's/^[[:space:]]*[AMD?]//' || true)
    SCAN_TYPE="uncommitted files"
fi

if [[ -z "$FILES_TO_SCAN" ]]; then
    echo -e "${GREEN}✅ No $SCAN_TYPE to audit${NC}"
    exit 0
fi

FILE_COUNT=$(echo "$FILES_TO_SCAN" | wc -l | tr -d ' ')
VIOLATION_COUNT=0

for file in $FILES_TO_SCAN; do
    if [[ -f "$file" ]]; then
        COUNT=$(awk 'length($0) > 100' "$file" | wc -l | tr -d ' ')
        VIOLATION_COUNT=$((VIOLATION_COUNT + COUNT))
    fi
done

if [[ $VIOLATION_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✅ $FILE_COUNT $SCAN_TYPE - No violations${NC}"
else
    echo -e "${YELLOW}⚠️  $FILE_COUNT $SCAN_TYPE - $VIOLATION_COUNT violations${NC}"
    echo -e "   Run: ${YELLOW}/audit${NC} for details"
fi
