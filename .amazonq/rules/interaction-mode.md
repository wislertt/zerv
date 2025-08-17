# Interaction Mode Rules

## CRITICAL: D: PREFIX ENFORCEMENT

**ABSOLUTE RULE - NO EXCEPTIONS:**

When user message starts with `D:` → **DISCUSSION ONLY MODE**

- NO code changes or modifications
- NO file writing (fsWrite, fsReplace)
- NO modification bash commands (rm, mv, cp, git commit, etc.)
- Reading tools OK (fsRead, listDirectory, executeBash for info) for context
- ONLY provide discussion, advice, and explanations

**IMPLEMENTATION MODE (Default):**

- No prefix = Use tools and make code changes as needed

## Examples

**Discussion Mode:**

- `D: How should we handle error cases?` → Only discuss approaches
- `D: What do you think about this design?` → Only provide opinions

**Implementation Mode:**

- `Add error handling to the git function` → Make actual code changes
- `Fix the bug in main.rs` → Use tools to implement fix

## ENFORCEMENT

The user is "tedious" about D: prefix compliance. Breaking this rule is unacceptable.

**IF MESSAGE STARTS WITH `D:`:**

1. Check first - does message start with `D:`?
2. If YES → No code changes, only discussion
3. Reading for context is allowed
4. **MUST include `[discussion mode]` tag in response**
5. Never break this rule
