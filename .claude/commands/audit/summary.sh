#!/bin/bash

# Quick summary of audit status
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if ! command -v git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Not in git repository${NC}"
    exit 1
fi

UNCOMMITTED_FILES=$(git status --porcelain | grep -E '^.*\.rs$' | sed 's/^[[:space:]]*[AMD?]//' | grep '\.rs$' || true)

if [[ -z "$UNCOMMITTED_FILES" ]]; then
    echo -e "${GREEN}✅ No uncommitted Rust files to audit${NC}"
    exit 0
fi

FILE_COUNT=$(echo "$UNCOMMITTED_FILES" | wc -l | tr -d ' ')
VIOLATION_COUNT=0

for file in $UNCOMMITTED_FILES; do
    if [[ -f "$file" ]]; then
        COUNT=$(awk 'length($0) > 100' "$file" | wc -l | tr -d ' ')
        VIOLATION_COUNT=$((VIOLATION_COUNT + COUNT))
    fi
done

if [[ $VIOLATION_COUNT -eq 0 ]]; then
    echo -e "${GREEN}✅ $FILE_COUNT uncommitted files - No violations${NC}"
else
    echo -e "${YELLOW}⚠️  $FILE_COUNT uncommitted files - $VIOLATION_COUNT violations${NC}"
    echo -e "   Run: ${YELLOW}/audit${NC} for details"
fi
