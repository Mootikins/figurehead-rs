# Figurehead Improvement Loop

You are working through TASKS.md using TDD methodology.

## Your Mission

1. Read TASKS.md to find the current phase and next uncompleted task
2. Work on ONE task at a time following TDD:
   - Write/verify failing test first (if applicable)
   - Implement minimal code to pass
   - Refactor if needed
3. Mark task complete `[x]` when done, update Progress Notes
4. Move to next task in sequence
5. When a PHASE is complete, STOP and announce it

## Rules

### TDD is Non-Negotiable
- Tests BEFORE implementation
- Red → Green → Refactor
- No skipping tests for "simple" changes

### Task Flow
```
[ ] pending     → [/] in_progress → [x] completed
                → [-] blocked (document why)
```

### Progress Tracking
- Update TASKS.md checkboxes as you work
- Append observations to "Progress Notes" section
- Commit after each completed task or logical group

### Verification
After each task: `just ci` must pass (clippy + tests)

### When to Stop

**MANDATORY STOP at phase boundaries.** Each phase ends with QA tasks and a "STOP HERE" marker. When you complete the QA section of a phase, you MUST stop for human review.

**STOP and output promise when:**
- You reach a **STOP HERE** marker (phase QA complete)
- You encounter a blocking issue requiring human input
- Context is getting long (you'll feel it)

**Promise format:**
```
<promise>PHASE 1 COMPLETE</promise>
```
or
```
<promise>BLOCKED: reason</promise>
```

**Do NOT proceed to the next phase without human approval.**

## Current State

Read TASKS.md now. Find the first `[ ]` task in the earliest incomplete phase. That's your target.

## Begin

1. `cat TASKS.md` to see current state
2. Find next task
3. Execute TDD cycle
4. Update TASKS.md
5. Verify with `just ci`
6. Commit if appropriate
7. Continue or stop at phase boundary
