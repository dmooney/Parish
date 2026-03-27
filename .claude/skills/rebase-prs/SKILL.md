---
name: rebase-prs
description: Monitor main for updates and rebase all open PRs. Use with /loop for continuous monitoring.
argument-hint: [--force]
---

Automatically rebase all open PRs onto the latest `origin/main`. Designed for one-shot use or continuous monitoring via `/loop 5m /rebase-prs`.

## Step 1: Detect changes on main

1. Record the current `origin/main` SHA: run `git rev-parse origin/main` and save the result as `OLD_SHA`.
2. Fetch the latest main: run `git fetch origin main`.
3. Record the new SHA: run `git rev-parse origin/main` and save as `NEW_SHA`.
4. If `$ARGUMENTS` contains `--force`, skip the comparison and proceed regardless.
5. If `OLD_SHA == NEW_SHA` (and no `--force`), report **"main unchanged, nothing to do"** and stop.
6. Otherwise, log: **"main updated: OLD_SHA → NEW_SHA"** and proceed.

## Step 2: List open PRs

Run:
```
gh pr list --state open --base main --json number,headRefName,title
```

- If the list is empty, report **"No open PRs targeting main"** and stop.
- Otherwise, save the list of `{number, headRefName, title}` for processing.

## Step 3: Rebase each PR

Process PRs **sequentially** (not in parallel — worktrees share `.git` state).

For each PR, use the **Agent tool** with `isolation: "worktree"` and provide it this task:

> Rebase PR #NUMBER (branch `BRANCH_NAME`) onto `origin/main`.
>
> 1. Run `git fetch origin main BRANCH_NAME`
> 2. Run `git checkout BRANCH_NAME`
> 3. Run `git rebase origin/main`
> 4. **If the rebase has conflicts:**
>    - Run `git diff --name-only --diff-filter=U` to list conflicted files
>    - For each conflicted file, read it and resolve the conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`) using your best judgment:
>      - Preserve the intent of both the PR changes and the main branch changes
>      - When in doubt, prefer the PR's version but incorporate main's structural changes
>    - Stage resolved files with `git add`
>    - Run `git rebase --continue`
>    - Repeat if more commits conflict (up to 10 rounds max)
>    - If a conflict is truly unresolvable (e.g., >50% of the file is conflicted with incompatible changes), run `git rebase --abort` and report FAILED
> 5. Run `cargo build` as a sanity check
> 6. If the build **passes**: run `git push origin BRANCH_NAME --force-with-lease`
> 7. If the build **fails**: run `git rebase --abort` (if still in progress), do NOT push, and report FAILED with the build error
>
> Return a structured result:
> - `status`: SUCCESS | PARTIAL | FAILED
> - `conflicts_resolved`: list of file paths that had conflicts (empty if clean rebase)
> - `build_passed`: true/false
> - `error`: description if failed, empty otherwise

## Step 4: Comment on each PR

After processing each PR, post a comment using `gh pr comment NUMBER --body "..."`.

**On SUCCESS or PARTIAL (conflicts resolved by AI):**

```markdown
## Automatic Rebase onto `main`

**Status**: SUCCESS (or: PARTIAL — conflicts resolved by AI)

| Detail | Value |
|--------|-------|
| Main updated | `OLD_SHA` → `NEW_SHA` |
| Branch | `BRANCH_NAME` |
| Build check | Passed |

**Conflicts resolved** (if any):
- `path/to/file.rs`: resolved overlapping changes

---
*Automated by Claude Code `/rebase-prs`*
```

**On FAILED:**

```markdown
## Automatic Rebase onto `main` — FAILED

| Detail | Value |
|--------|-------|
| Main updated | `OLD_SHA` → `NEW_SHA` |
| Branch | `BRANCH_NAME` |
| Failure reason | (description) |

**Action required**: Please rebase this PR manually.

---
*Automated by Claude Code `/rebase-prs`*
```

## Step 5: Summary

Print a summary table to the user:

```
PR #  | Branch              | Status  | Conflicts
------|---------------------|---------|----------
#42   | feat/new-feature    | SUCCESS | none
#38   | fix/broken-thing    | PARTIAL | 2 files
#35   | refactor/cleanup    | FAILED  | unresolvable
```
