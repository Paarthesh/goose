---
name: test-driven-development
description: Red-green-refactor cycle — write the failing test first, make it pass with minimal code, then refactor cleanly
---

# Test-Driven Development

**Apply when:** You are adding a new function, fixing a bug, or implementing a feature where the correct behavior can be specified as a test before the implementation exists. Use TDD by default for logic-heavy code, parsers, state machines, and any code with non-trivial edge cases.

---

## Step 1: Write the test first (RED)

Before touching production code, write a test that:
- Names the behavior being tested, not the implementation (`test_parse_empty_input_returns_error`, not `test_parse1`)
- Has one logical assertion per test
- Fails for the right reason (not a compile error — a semantic failure)

```rust
#[test]
fn parse_empty_input_returns_error() {
    let result = parse("");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "input must not be empty");
}
```

Run the test and confirm it **fails**:
```bash
cargo test -p <crate> parse_empty_input_returns_error
```

If it passes without implementation, the test is wrong — fix the test.

## Step 2: Make it pass with minimal code (GREEN)

Write the **smallest production code** that makes the test pass. Do not generalize yet. Do not add features not required by the current test. Premature generalization is the enemy of TDD.

```rust
pub fn parse(input: &str) -> Result<Ast, ParseError> {
    if input.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    todo!("implement the rest")
}
```

Run the test and confirm it **passes**:
```bash
cargo test -p <crate> parse_empty_input_returns_error
```

## Step 3: Refactor (REFACTOR)

Now that the test is green, clean up:
- Extract repeated logic
- Rename for clarity
- Remove duplication between the new code and existing code
- Apply project conventions

After each refactor step, run the full test suite:
```bash
cargo test -p <crate>
```

The test must remain green throughout. If it goes red, undo the last refactor step.

## Step 4: Repeat for the next behavior

Add the next test for the next behavior. Each cycle — RED → GREEN → REFACTOR — handles one behavior. Do not add multiple behaviors to a single cycle.

Typical cycle order: happy path → error cases → edge cases → integration.

## Step 5: Run the full suite before committing

```bash
cargo test -p <crate>
cargo clippy --all-targets -- -D warnings
```

All tests must pass, including pre-existing ones.

---

**Done when:** All specified behaviors have passing tests, no existing tests are broken, and the implementation is clean (no dead code, no TODOs in shipped code).
