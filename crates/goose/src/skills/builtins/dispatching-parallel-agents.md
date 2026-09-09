---
name: dispatching-parallel-agents
description: Fan out to parallel agents — decompose tasks by independence, scope context tightly per agent, integrate and reconcile results
---

# Dispatching Parallel Agents

**Apply when:** Work can be divided into two or more tasks that have no data dependency on each other, the total work would saturate a single context window, or you need concurrent exploration of different parts of a codebase.

---

## Step 1: Identify true independence

Two subtasks are independent if:
- Subtask B does not need any output from subtask A to begin
- Subtask A does not need to write to a file that subtask B reads

If there is a dependency, dispatch subtasks sequentially: A → collect result → B.

Rule of thumb: exploration tasks (read-only) are almost always parallelizable. Write tasks (edit files, run commands) require careful sequencing to avoid conflicts.

## Step 2: Decompose with clean boundaries

Good decomposition:
- Each agent works in a different area of the codebase
- Each agent produces a self-contained artifact (a report, a list, a file)
- Each agent's scope is stated as a file path or module, not a vague description

Bad decomposition:
- "Agent A does the hard part, Agent B does the rest" (ambiguous boundary)
- Two agents that both need to edit the same file (write conflict)

## Step 3: Write minimal, complete prompts

Each agent starts with zero memory of this conversation. Its prompt must be self-contained:

```
You are working in the goose codebase at /home/user/goose.

Task: List every place where `ExtensionConfig::Stdio` is constructed.
Scope: Search only crates/goose/ and crates/goose-cli/. Ignore test files.

Output format:
| File | Line | Code snippet (one line) |
(markdown table)

Do not make any edits. Return only the table.
```

Include:
- The repo path
- The exact task (what to find, what to produce)
- The scope (what to search, what to ignore)
- The output format (so you can parse the result without ambiguity)

## Step 4: Dispatch all independent agents in a single turn

Spawn all agents in one response — not one per turn. Sequential spawning serializes work that could run concurrently.

Assign a short label to each agent in your dispatch message so you can track which result came from which agent.

## Step 5: Validate each result on receipt

When an agent's result arrives:
- Does it match the requested output format?
- Is any scope boundary visibly violated (e.g., it searched outside the specified directories)?
- Does it report uncertainty or say "I could not find X"?

If a result is incomplete or malformed, send a corrective follow-up to that agent rather than discarding and restarting.

## Step 6: Integrate results

Combine the agents' outputs into a single coherent answer:
- Deduplicate overlapping findings
- Resolve contradictions: state which source is correct and why
- Flag missing coverage: if Agent A covered `crates/goose/` and Agent B covered `crates/goose-cli/`, confirm `crates/goose-server/` was not needed or dispatch a third agent for it

## Step 7: Validate the integrated result yourself

Before presenting the integrated result to the user or using it to drive the next step, do a consistency pass yourself (not via subagent):
- Are there gaps in coverage?
- Do any findings contradict each other in a way you have not resolved?
- Is the combined output internally consistent?

---

**Done when:** All agents have returned results, every result has been validated, the integrated output is internally consistent, and you have confirmed no scope gaps remain.
