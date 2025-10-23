#!/bin/bash

# Main audit script - checks uncommitted files for all violations
set -euo pipefail

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

if ! command -v git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

echo -e "${BLUE}🔍 Zerv Code Quality Audit${NC}"
echo -e "${BLUE}=========================${NC}"
echo ""

# Get uncommitted files
UNCOMMITTED_FILES=$(git status --porcelain | grep -E '^.*\.rs$' | sed 's/^[[:space:]]*[AMD?]//' | grep '\.rs$' || true)

if [[ -z "$UNCOMMITTED_FILES" ]]; then
    echo -e "${GREEN}✅ No uncommitted Rust files found${NC}"
    exit 0
fi

echo -e "${YELLOW}📁 Scanning uncommitted files:${NC}"
echo "$UNCOMMITTED_FILES" | sed 's/^/  - /'
echo ""

TOTAL_VIOLATIONS=0
LONG_LINES=0
COMMENT_VIOLATIONS=0
IMPORT_VIOLATIONS=0

# Check each file
for file in $UNCOMMITTED_FILES; do
    if [[ ! -f "$file" ]]; then
        continue
    fi

    echo -e "${BLUE}🔬 Checking: ${file}${NC}"

    file_violations=0

    # Long lines
    while IFS= read -r line; do
        if [[ -n "$line" ]]; then
            ((LONG_LINES++))
            ((file_violations++))
            echo -e "  ${RED}Line ${line}${NC}"
        fi
    done < <(awk 'length($0) > 100 {print NR ": [" length($0) " chars] " $0}' "$file" || true)

    # Comment violations
    comment_issues=$(grep -n "// Initialize\|// Create\|// Return\|// Calculate\|// Format\|// ====\|// ----" "$file" || true)
    if [[ -n "$comment_issues" ]]; then
        ((COMMENT_VIOLATIONS++))
        ((file_violations++))
        echo -e "  ${RED}Bad comment patterns:${NC}"
        echo "$comment_issues" | sed 's/^/    /'
    fi

    # Inline imports
    import_issues=$(grep -n "fn.*{[^}]*use " "$file" || true)
    if [[ -n "$import_issues" ]]; then
        ((IMPORT_VIOLATIONS++))
        ((file_violations++))
        echo -e "  ${RED}Inline imports:${NC}"
        echo "$import_issues" | sed 's/^/    /'
    fi

    if [[ $file_violations -eq 0 ]]; then
        echo -e "  ${GREEN}✅ No violations found${NC}"
    fi

    ((TOTAL_VIOLATIONS += file_violations))
    echo ""
done

# Summary
echo -e "${BLUE}📊 Summary${NC}"
echo -e "${BLUE}-----------${NC}"
echo -e "Files scanned: $(echo "$UNCOMMITTED_FILES" | wc -l | tr -d ' ')"
echo -e "Long line violations: ${RED}$LONG_LINES${NC}"
echo -e "Comment violations: ${RED}$COMMENT_VIOLATIONS${NC}"
echo -e "Import violations: ${RED}$IMPORT_VIOLATIONS${NC}"
echo -e "Total violations: ${RED}$TOTAL_VIOLATIONS${NC}"
echo ""

if [[ $TOTAL_VIOLATIONS -gt 0 ]]; then
    echo -e "${YELLOW}💡 Fix suggestions:${NC}"
    echo -e "  • Use format!() for long command strings"
    echo -e "  • Break rstest attributes across multiple lines"
    echo -e "  • Extract complex strings to variables"
    echo -e "  • Move use statements to file top"
    echo ""
    echo -e "${RED}❌ Audit failed - $TOTAL_VIOLATIONS violations found${NC}"
    echo -e "   Run: ${BLUE}cat .claude/commands/audit.md${NC} for fix examples"
    exit 1
else
    echo -e "${GREEN}✅ Audit passed - No violations found${NC}"
fi
