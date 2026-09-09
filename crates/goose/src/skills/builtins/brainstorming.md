---
name: brainstorming
description: Diverge-converge ideation before committing to an approach — enumerate options, stress-test each, pick the best fit for the constraints
---

# Brainstorming

**Apply when:** You are about to make a non-trivial design or implementation choice, the user asks "how should I…", or you realize mid-task that there are multiple valid approaches worth comparing before committing.

---

## Step 1: Clarify the goal and constraints

Before generating ideas, state what you are optimizing for:
- What is the outcome? (correctness, speed, simplicity, reversibility, …)
- What are the hard constraints? (existing codebase patterns, dependencies, performance budget, …)
- What is out of scope?

If any constraint is unclear, ask one focused question now — not during implementation.

## Step 2: Diverge — generate at least 3 options

Generate **at least three** distinct approaches. Do not evaluate yet. Label them:

```
Option A: <name>
  Idea: <one sentence>

Option B: <name>
  Idea: <one sentence>

Option C: <name>
  Idea: <one sentence>
```

Include at least one unconventional option. If the obvious answer is A, force yourself to articulate B and C anyway — one of them frequently turns out to be better.

## Step 3: Stress-test each option

For each option, identify:
- **Strongest argument for it**
- **Worst failure mode** (what breaks or becomes hard to change later?)
- **Hidden cost** (complexity, coupling, performance, maintenance burden)

Use concrete examples, not abstract adjectives. "This couples the parser to the storage layer, making it impossible to test without a database" beats "this is tightly coupled."

## Step 4: Converge — pick one and commit

Choose the option that best satisfies the goal under the constraints. State:
```
Decision: Option <X>
Reason: <1-2 sentences tying back to constraints from Step 1>
Not chosen: Option Y because <one concrete reason>; Option Z because <one concrete reason>
```

Do not hedge with "either could work." Pick one and own it.

## Step 5: Surface the decision

Write the decision at the top of your next message or plan so the user can redirect before you implement. A decision they can see and override beats a silent assumption they discover in code review.

---

**Done when:** You have written down a decision with the reason it was chosen and why the alternatives were not, and the user has not redirected.
