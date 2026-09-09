---
name: finishing-a-development-branch
description: Branch completion playbook — final checks, PR creation or merge, worktree cleanup, and clean discard if needed
---

# Finishing a Development Branch

**Apply when:** You have completed work on a feature or fix branch and need to merge it, open a PR, or cleanly discard it. Also apply when cleaning up after a merged PR.

---

## Step 1: Final pre-merge checklist

Before any merge or PR action:

```bash
# 1. All tests pass
cargo test -p <crate>

# 2. No clippy warnings
cargo clippy --all-targets -- -D warnings

# 3. Formatting is clean
cargo fmt --check

# 4. Review your own diff one more time
git diff main...HEAD -- . ':(exclude)Cargo.lock'
```

Look for: debug prints, TODOs, commented-out code, unintended file changes, missing file additions.

## Step 2a: Open a PR (for shared/reviewed work)

```bash
# Push the branch
git push -u origin <branch-name>
```

PR description must include:
- **What changed** (1 paragraph, not a list of commits)
- **Why** (the problem or requirement it addresses)
- **How to verify** (the specific manual step a reviewer can take)
- **Checklist** (build passes, tests pass, clippy clean)

Check the repo for a PR template (`.github/pull_request_template.md`) and follow its structure.

## Step 2b: Merge directly (for solo work on your own fork)

```bash
git checkout main
git merge --no-ff <branch-name> -m "feat: <description>"
git push origin main
```

Use `--no-ff` to preserve branch history.

## Step 2c: Discard the branch (if the approach was wrong)

If the work is being abandoned:

```bash
# Delete local branch
git checkout main
git branch -D <branch-name>

# Delete remote branch (if pushed)
git push origin --delete <branch-name>
```

Stash any ideas worth preserving in a comment or note before deleting.

## Step 3: Clean up worktrees (if used)

If you created a worktree for this branch:

```bash
# From the main repo directory:
git worktree remove ../<worktree-dir>

# If removal fails due to uncommitted changes you want to discard:
git worktree remove --force ../<worktree-dir>

# Prune any stale metadata:
git worktree prune
```

Verify:
```bash
git worktree list   # should show only the expected worktrees
```

## Step 4: Delete the branch after merge

After a PR is merged or a direct merge is complete:

```bash
git branch -d <branch-name>           # local (safe — checks merge status)
git push origin --delete <branch-name> # remote
```

Use `-D` instead of `-d` only if you are certain the branch is fully merged and the history is preserved in the target branch.

## Step 5: Verify the target branch is clean

```bash
git checkout main
git pull origin main
cargo build       # confirm main still builds
cargo test -p <crate>  # confirm tests still pass on main
```

---

**Done when:** The branch is merged or discarded, all worktrees for it are removed, the local and remote branch references are deleted, and the target branch builds and tests cleanly.
