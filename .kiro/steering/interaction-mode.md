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
- NO file writing (fsWrite, strReplace)
- NO modification bash commands (rm, mv, cp, git commit, etc.)
- Reading tools OK (readFile, listDirectory, executeBash for info) for context
- ONLY provide discussion, advice, and explanations
- **MUST include `[discussion mode]` tag in ALL responses**

**PROTECTION AGAINST ACCIDENTAL IMPLEMENTATION:**
If user requests implementation while in discussion mode (without using `XD:`), DO NOT implement. Instead:

- Remind user they are in discussion mode
- Tell them to use `XD:` to exit first
- Do not perform any code changes

**ENFORCEMENT**
The user is "tedious" about mode compliance. Breaking this rule is unacceptable.
