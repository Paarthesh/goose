---
name: verification-before-completion
description: Mandatory checklist to run before marking any task done — build, test, lint, diff review, and smoke test
---

# Verification Before Completion

**Apply when:** You are about to declare a task complete, push a commit, or hand work back to the user. Skipping this skill is the most common source of "it looked right but broke CI."

---

## Step 1: Build

```bash
cargo build
```

A debug build catches type errors and import issues. Fix all errors before proceeding.

If the change touches a binary or server:
```bash
cargo build --release   # catches release-mode issues (e.g. optimizations that expose UB)
```

## Step 2: Test

Run the tests most directly related to your change first, then the full suite:

```bash
# Targeted (fast):
cargo test -p <crate> <module_or_test_name>

# Full suite for the affected crate:
cargo test -p <crate>

# If the change touches multiple crates:
cargo test
```

Do not declare success until all tests pass. A test failure is a blocker, not a warning.

## Step 3: Lint

```bash
cargo clippy --all-targets -- -D warnings
```

Zero warnings. If a clippy lint is a false positive, add `#[allow(...)]` with a comment explaining why — do not disable the check globally.

```bash
cargo fmt --check
```

If this fails, run `cargo fmt` and re-verify.

## Step 4: Review your own diff

```bash
git diff HEAD~1..HEAD    # if committed
git diff                 # if not yet committed
```

Read every line. Look for:
- `dbg!`, `println!`, `eprintln!` debug prints
- `todo!()`, `unimplemented!()`, `panic!("TODO")` in non-test code
- Commented-out blocks left over from debugging
- Files that should not be changed (generated files, lockfiles edited by hand)
- Missing `mod` declarations for new files
- Sensitive values hard-coded (tokens, passwords, internal URLs)

## Step 5: Smoke test (if applicable)

For user-facing changes, run the actual binary or UI with the new code:

```bash
cargo run --bin goose -- <relevant-subcommand>
```

Confirm the specific behavior you changed works end-to-end. A passing test suite does not replace a smoke test — tests can be wrong.

## Step 6: Check the commit message

The commit message must:
- Start with a type prefix: `feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`
- Describe what changed and why (not "updated code")
- Include `git commit -s` (DCO sign-off required per AGENTS.md)

---

**Done when:** All six steps complete without errors or findings that need resolution. Only then is the task complete.
