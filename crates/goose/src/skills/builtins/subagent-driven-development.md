---
name: subagent-driven-development
description: Decompose work into isolated subtasks, spawn subagents with scoped context, collect results, and integrate them cleanly
---

# Subagent-Driven Development

**Apply when:** A task has two or more clearly independent subtasks that can run concurrently, the total work would fill a single context window, or you need specialized exploration across multiple areas of the codebase simultaneously.

---

## Step 1: Decompose into independent subtasks

A subtask is a good candidate for a subagent when:
- Its inputs are fully known upfront (no dependency on another subtask's output)
- Its output is a well-defined artifact (a file, a report, a list of findings)
- Failure in one subtask should not block the others

Bad decomposition: subtask B needs subtask A's output. Spawn A first, get the result, then spawn B.

Good decomposition: subtask A explores `crates/goose/src/agents/`, subtask B explores `crates/goose-mcp/src/`. Both can run in parallel.

## Step 2: Write a tight prompt for each subagent

Each subagent starts with an empty context. Give it everything it needs in its prompt:

```
You are exploring the goose codebase at /home/user/goose.

Task: Find all callers of `ExtensionManager::add_extension` and report:
- File path and line number
- What ExtensionConfig variant they pass
- Whether they handle the returned error

Search in crates/goose/ and crates/goose-cli/. Ignore test files.

Return a markdown table: | File | Line | Config variant | Error handled? |
```

Do NOT assume the subagent has memory of this conversation. Be explicit about paths, scope, and output format.

## Step 3: Dispatch agents in parallel

Spawn all independent agents in a single turn (one tool call per agent). Do not wait for one to finish before starting the next — that serializes work that could run concurrently.

Label each agent with a short description so you can identify their outputs:
- Agent A: "Explore extension callers in goose-cli"
- Agent B: "Explore extension callers in goose-server"

## Step 4: Collect and validate results

When agents complete, read each result before integrating:
- Does the output match the requested format?
- Are there contradictions between agents that need reconciliation?
- Did any agent report uncertainty or missing information?

If an agent's result is incomplete, send a follow-up with the specific missing piece rather than re-running the full task.

## Step 5: Integrate results

Merge the outputs into a coherent whole. Resolve contradictions explicitly — state which source you trusted and why. Do not silently average conflicting findings.

If integration reveals that you need another round of subagent work (e.g., a finding from Agent A changes what Agent B needs to explore), spawn the follow-up agents now.

## Step 6: Verify the integrated result

After integration, do a final consistency check yourself — not via subagent — to confirm the combined output is internally consistent and answers the original task.

---

**Done when:** All subtask results are collected, integrated into a coherent output, and verified for internal consistency.
