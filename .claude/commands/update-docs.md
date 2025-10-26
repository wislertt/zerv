Automated documentation maintenance workflow for keeping llms.md synchronized with codebase changes.

## Process

1. Detect documentation-relevant changes since last marker timestamp
2. Generate changelog summary in temporary cache directory
3. Analyze changes for CLI relevance and impact
4. Update llms.md based on changelog analysis
5. Refresh marker timestamp with current time

## Implementation

Use the following workflow to update documentation:

```bash
# Check if marker file exists
if [ ! -f "docs/.last-update" ]; then
    echo "No marker file found, creating initial timestamp"
    date -u +"%Y-%m-%dT%H:%M:%SZ" > docs/.last-update
    exit 0
fi

# Get last marker commit hash
MARKER_COMMIT=$(git log -1 --format=%H docs/.last-update 2>/dev/null || echo "")
CURRENT_COMMIT=$(git rev-parse HEAD)

# Generate changelog for changes since marker
mkdir -p docs/.cache
if [ -n "$MARKER_COMMIT" ] && [ "$MARKER_COMMIT" != "$CURRENT_COMMIT" ]; then
    git diff "$MARKER_COMMIT..HEAD" --name-status > docs/.cache/CHANGES.md
    git diff "$MARKER_COMMIT..HEAD" >> docs/.cache/CHANGES.md
elif [ -z "$MARKER_COMMIT" ]; then
    echo "First run - checking recent changes" > docs/.cache/CHANGES.md
    git log --oneline -10 >> docs/.cache/CHANGES.md
fi

# Include uncommitted changes
git diff HEAD >> docs/.cache/CHANGES.md 2>/dev/null || true

# Analyze changes for CLI relevance
echo "=== CHANGE ANALYSIS ===" > docs/.cache/CHANGELOG.md
echo "" >> docs/.cache/CHANGELOG.md

# Check for high impact changes
HIGH_IMPACT_FILES=(
    "src/cli/parser.rs"
    "src/cli/app.rs"
    "src/cli/llm_help.rs"
)

for file in "${HIGH_IMPACT_FILES[@]}"; do
    if grep -q "$file" docs/.cache/CHANGES.md 2>/dev/null; then
        echo "HIGH IMPACT: $file modified" >> docs/.cache/CHANGELOG.md
    fi
done

# Check for new CLI options/commands
if grep -q "arg\|clap\|command\|parser" docs/.cache/CHANGES.md 2>/dev/null; then
    echo "MEDIUM IMPACT: CLI arguments/commands potentially modified" >> docs/.cache/CHANGELOG.md
fi

# Check for help text changes
if grep -q "help\|description" docs/.cache/CHANGES.md 2>/dev/null; then
    echo "MEDIUM IMPACT: Help text potentially modified" >> docs/.cache/CHANGELOG.md
fi

echo "" >> docs/.cache/CHANGELOG.md
echo "=== RAW CHANGES ===" >> docs/.cache/CHANGELOG.md
cat docs/.cache/CHANGES.md >> docs/.cache/CHANGELOG.md

# Display changelog for manual review
cat docs/.cache/CHANGELOG.md

echo ""
echo "Review the changes above. If llms.md needs updates:"
echo "1. Manually update docs/llms.md based on the analysis"
echo "2. Run: date -u +\"%Y-%m-%dT%H:%M:%SZ\" > docs/.last-update"
echo "3. Commit both the documentation changes and marker update"
```
