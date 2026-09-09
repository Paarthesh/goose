---
name: systematic-debugging
description: 4-phase root cause analysis — reproduce, isolate, hypothesize, verify — to escape random-change debugging
---

# Systematic Debugging

**Apply when:** A test fails, a bug is reported, unexpected behavior is observed, or you are tempted to make a change "and see if it helps." Random changes without root cause analysis waste time and introduce new bugs.

---

## Phase 1: Reproduce

**Goal:** Produce a minimal, reliable reproduction of the failure.

1. Run the failing test or reproduce the behavior in the smallest possible way:
   ```bash
   cargo test -p <crate> <test_name> -- --nocapture
   ```

2. Confirm the failure is consistent: run it 3 times. If it is intermittent, note the failure rate and check for concurrency issues.

3. Write down the exact failure:
   ```
   Expected: parse("2024-01-01") returns Ok(Date { year: 2024, month: 1, day: 1 })
   Actual:   parse("2024-01-01") returns Err(ParseError::InvalidFormat)
   ```

Do not proceed to Phase 2 until you have a reliable reproduction. Debugging an unreliable reproduction is random work.

## Phase 2: Isolate

**Goal:** Find the smallest code path that exhibits the failure.

1. **Bisect if recent:** If the code was working before, use `git bisect` to find the commit that introduced the bug:
   ```bash
   git bisect start
   git bisect bad HEAD
   git bisect good <last-known-good-commit>
   # git bisect run cargo test -p <crate> <test_name>
   ```

2. **Narrow by commenting out / simplifying:** Remove branches and inputs not related to the failure path. A 500-line function with a bug can often be reduced to 20 lines.

3. **Add targeted logging:** Instrument the specific path with `eprintln!` or `tracing::debug!` — not broad logging, but focused on the variables involved in the failure.
   ```rust
   eprintln!("DEBUG parse: input={:?} state={:?}", input, state);
   ```

4. **State the isolated location:** "The bug is in `parser.rs:142`, inside the `parse_date` function, specifically in the month boundary check."

## Phase 3: Hypothesize

**Goal:** Form a specific, falsifiable hypothesis about the root cause.

Write down:
```
Hypothesis: The month boundary check at parser.rs:142 uses > instead of >=,
  which means day 31 of a 31-day month is incorrectly rejected.
```

A good hypothesis is:
- **Specific** (names the exact line, variable, or condition)
- **Falsifiable** (you can write a test that would prove it wrong)
- **Explains all observed symptoms** (not just the one you are looking at)

If your hypothesis does not explain all symptoms, it is incomplete.

## Phase 4: Verify

**Goal:** Confirm the hypothesis before fixing.

1. Write a test that should pass if the hypothesis is correct and currently fails:
   ```rust
   #[test]
   fn parse_last_day_of_31_day_month() {
       assert!(parse("2024-01-31").is_ok());  // currently fails
   }
   ```

2. Apply the minimal fix implied by the hypothesis:
   ```rust
   // Before: if day > month_length(month) {
   if day > month_length(month) {  // change > to >=? No — check the actual logic
   ```

3. Confirm the fix makes the new test pass **and** all existing tests still pass:
   ```bash
   cargo test -p <crate>
   ```

4. If the fix does not make the test pass, the hypothesis was wrong. Return to Phase 2 with the new information.

---

**Done when:** The root cause is identified at a specific location, a targeted fix is applied, the failing test now passes, and no existing tests are broken. The fix is the minimal change required — not a broader refactor.
