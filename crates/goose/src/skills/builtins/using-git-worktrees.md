---
name: using-git-worktrees
description: Create, use, and clean up git worktrees for parallel branch work without stashing or context-switching overhead
---

# Using Git Worktrees

**Apply when:** You need to work on two branches simultaneously (e.g., a hotfix while a feature is in progress), you want to review a PR branch without disturbing your working tree, or a subagent needs an isolated checkout on a different branch.

---

## Step 1: Understand the current repo state

```bash
git worktree list          # see all existing worktrees
git branch -a              # see available branches
```

The main worktree is the repo root. Never create a worktree inside an existing worktree directory.

## Step 2: Create a worktree

**For an existing branch:**
```bash
git worktree add ../goose-hotfix hotfix/critical-bug
```

**For a new branch:**
```bash
git worktree add -b feature/new-thing ../goose-feature main
```

The second argument is the **filesystem path** for the new worktree (typically a sibling directory). The third argument is the base ref.

Each worktree is a full working directory with its own index — you can build, test, and edit independently.

## Step 3: Work in the worktree

```bash
cd ../goose-hotfix
# make changes, run tests, commit as normal
cargo test -p goose
git add -p
git commit -s -m "fix: ..."
```

Changes in one worktree do not affect other worktrees. You can have both open in separate terminal sessions simultaneously.

## Step 4: Push from the worktree

```bash
git push -u origin hotfix/critical-bug
```

Pushes from a worktree work identically to the main repo.

## Step 5: Clean up after merging

Once the branch is merged (or discarded), remove the worktree:

```bash
# From the main repo directory:
git worktree remove ../goose-hotfix

# If the worktree has uncommitted changes and you are sure you want to discard:
git worktree remove --force ../goose-hotfix

# Prune stale worktree metadata (if directory was deleted manually):
git worktree prune
```

Then delete the branch if it is no longer needed:
```bash
git branch -d hotfix/critical-bug
git push origin --delete hotfix/critical-bug   # if pushed
```

## Common pitfalls

- **"already checked out"**: a branch can only be checked out in one worktree at a time. Use `git worktree list` to find where it is.
- **Cargo target sharing**: worktrees in sibling directories share the workspace `target/` by default — this is intentional and speeds up builds.
- **Absolute paths in tools**: some tools hard-code the repo root. Run them from the worktree directory.

---

**Done when:** The worktree is removed, the branch is deleted or merged, and `git worktree list` shows only the expected worktrees.
