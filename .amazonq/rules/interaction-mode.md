# Interaction Mode Rules

## CRITICAL: STICKY DISCUSSION MODE

**ABSOLUTE RULE - NO EXCEPTIONS:**

**ENTRY TRIGGERS (Enter Discussion Mode):**

- `D:` at start of message → Enter **DISCUSSION ONLY MODE**
- Mode persists across ALL subsequent messages

**EXIT TRIGGER (Return to Implementation Mode):**

- `XD:` at start of message → Exit discussion mode, return to implementation

**DISCUSSION MODE BEHAVIOR:**

- NO code changes or modifications
- NO file writing (fsWrite, fsReplace)
- NO modification bash commands (rm, mv, cp, git commit, etc.)
- Reading tools OK (fsRead, listDirectory, executeBash for info) for context
- ONLY provide discussion, advice, and explanations
- **MUST include `[discussion mode]` tag in ALL responses**

**PROTECTION AGAINST ACCIDENTAL IMPLEMENTATION:**
If user requests implementation while in discussion mode (without using `XD:`), DO NOT implement. Instead:

- Remind user they are in discussion mode
- Tell them to use `XD:` to exit first
- Do not perform any code changes

**IMPLEMENTATION MODE (Default):**

- No prefix = Use tools and make code changes as needed

## Examples

**Entering Discussion Mode:**

- `D: How should we handle error cases?` → Enter discussion mode
- Next message: `What about performance?` → Still in discussion mode
- Next message: `XD: Implement the solution` → Exit and implement

**Accidental Implementation Request:**

- In discussion mode: `Add error handling to main.rs`
- Response: `[discussion mode] You're currently in discussion mode. Use XD: to exit first if you want me to implement changes.`

## ENFORCEMENT

The user is "tedious" about mode compliance. Breaking this rule is unacceptable.

**WORKFLOW:**

1. Track conversation state - am I in discussion mode?
2. If in discussion mode: ONLY discuss, never implement
3. If implementation requested without `XD:`: Remind about exit trigger
4. Always show `[discussion mode]` tag when active
