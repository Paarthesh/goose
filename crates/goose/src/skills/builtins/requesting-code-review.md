---
name: requesting-code-review
description: Request a review focused on correctness and critical issues, triage findings by severity, and apply or rebut each one
---

# Requesting Code Review

**Apply when:** You have completed an implementation and want to verify correctness before merging, or you suspect a logic error you cannot locate through testing alone.

---

## Step 1: Prepare the diff for review

Before requesting a review, ensure:
- All tests pass: `cargo test -p <crate>`
- No clippy warnings: `cargo clippy --all-targets -- -D warnings`
- No debug prints, commented-out code, or `todo!()` macros in production paths
- Commit history is clean (no "WIP" commits in the PR)

A reviewer's time is wasted on lint findings that should have been caught before review.

## Step 2: Frame the review request

State clearly:
- **What changed:** one-paragraph summary of the diff's purpose
- **What to focus on:** the areas of highest risk (complex logic, concurrency, error handling, security boundaries)
- **What to skip:** generated code, trivial formatting changes, boilerplate

Example:
```
Focus on:
- The lock ordering in `extension_manager.rs:add_extension` (potential deadlock)
- Error propagation in `route_handler.rs:handle_request` (is the ? correct at line 47?)

Skip:
- The OpenAPI JSON (generated, do not edit)
- Test file boilerplate
```

## Step 3: Triage findings by severity

When review findings arrive, classify each:

| Severity | Definition | Required action |
|---|---|---|
| **Critical** | Correctness bug, data loss, security flaw, panic in production path | Must fix before merge |
| **Major** | Wrong behavior in edge case, API contract violation, memory leak | Fix before merge unless explicitly deferred |
| **Minor** | Style, naming, unnecessary complexity | Fix if cheap; comment explaining why not if skipped |
| **Nit** | Whitespace, typos, formatting | Fix or ignore; no comment needed |

Do not treat all findings as equally urgent. Fix criticals first.

## Step 4: Address each finding explicitly

For every Critical and Major finding:
- Either **apply the fix** and push a new commit
- Or **rebut it** with a specific technical argument

Silence is not a rebuttal. A finding left unaddressed is assumed accepted.

```
Finding: "This RwLock could deadlock if add_extension is called recursively"
Response: Fixed — moved the lock acquisition to after the async call at line 89.
  (or)
Response: Not a deadlock: add_extension is only called from the session loop which
  is single-threaded per session. Added a comment to document this invariant.
```

## Step 5: Request re-review after addressing criticals

After fixing Critical and Major findings, signal that they are addressed:
- Reference the specific finding in the commit message
- Comment on the review thread with what changed

Do not request a re-review until all Criticals are addressed.

---

**Done when:** All Critical findings are fixed or formally rebutted, all Major findings are addressed or deliberately deferred with a note, and the reviewer has acknowledged the responses.
