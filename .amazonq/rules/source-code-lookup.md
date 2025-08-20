# Source Code Lookup Rules

## MANDATORY: Check Local Cache First

When user asks to "look at source code" or examine external project implementations:

**ALWAYS check `.cache/repos/` directory first** before attempting external lookups.

## Workflow

1. **Check local cache**: Look in `.cache/repos/<project-name>/`
2. **If found**: Use local cached version
3. **If not found**: Then attempt external lookup (GitHub API, etc.)

## Examples

- "look at dunamai source code" → Check `.cache/repos/dunamai/` first
- "examine ripgrep implementation" → Check `.cache/repos/ripgrep/` first
- "see how X handles Y" → Check `.cache/repos/X/` first
