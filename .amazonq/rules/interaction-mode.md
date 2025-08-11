# Interaction Mode Rules

## MANDATORY: Prefix-Based Behavior Control

**STRICT RULE-BASED PREFIXES:**

- `D:` at start of message = Discussion only, no code changes, no tool calls
- No prefix = Default IMPLEMENT mode (make code changes and use tools as needed)

**Examples:**

- `D: How should we handle error cases in the git utilities?`
- `Add error handling to the git init function` (default IMPLEMENT mode)

This ensures predictable behavior and gives user precise control over whether code changes are made.
