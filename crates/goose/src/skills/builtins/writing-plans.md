---
name: writing-plans
description: Write an implementation plan — context, chosen approach, critical files, and verification steps — before touching code
---

# Writing Plans

**Apply when:** A task is complex enough that the wrong approach would waste significant effort, you are about to make changes across more than two files, or the user asks for a plan before implementation.

---

## Step 1: State the problem

Write a one-paragraph **Context** section that answers:
- What existing behavior or gap prompted this change?
- What is the desired outcome?
- What scope is explicitly out of scope?

This is for the reader (and your future self mid-implementation). If you cannot write this clearly, you do not yet understand the task — ask a clarifying question before proceeding.

## Step 2: Explore before designing

Do not write the approach section until you have read the relevant code. Identify:
- The entry point(s) affected
- Existing patterns, utilities, or abstractions you should reuse (not reinvent)
- Tests that already cover the area (so you know what must not break)

Reference specific file paths (`crates/goose/src/agents/agent.rs:142`) not vague descriptions ("somewhere in the agent code").

## Step 3: Write the chosen approach

One approach, not a menu. Structure:

```
## Approach

<2–4 sentences: what you will change and why this design>

### Files to modify
- `path/to/file.rs` — what changes and why
- `path/to/other.rs` — what changes and why

### New files
- `path/to/new.rs` — what it contains

### Functions/utilities to reuse
- `crates/goose/src/util.rs::helper_fn` — already does X, call it instead of reimplementing
```

Do not list every file that will be recompiled — only files you will edit or create.

## Step 4: Write the verification section

Before any implementation is "done", these checks must pass. List them explicitly:

```
## Verification
- [ ] cargo build succeeds
- [ ] cargo test -p <crate> (name the specific tests that exercise the change)
- [ ] cargo clippy --all-targets -- -D warnings
- [ ] Manual smoke test: <describe the specific action to verify end-to-end>
```

Vague items like "test it" are not verification steps.

## Step 5: Surface the plan before coding

Present the plan to the user (or write it to a file) and wait for a redirect signal. A plan the user can read costs one round-trip. A misguided implementation costs ten.

If the user approves (explicitly or implicitly by moving on), begin implementation immediately — do not re-explain the plan in prose.

---

**Done when:** The plan is written, the user has not redirected, and implementation has begun (or the user is reviewing).
