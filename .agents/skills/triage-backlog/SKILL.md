---
name: triage-backlog
description: Walk the open-issue backlog, classify each un-triaged issue by theme + priority (P0–P3), and apply labels via the GitHub MCP server. Use when the audit workflow flags un-triaged issues, after a batch of new issues lands, or anytime the user asks for triage.
---

Run a triage pass over open issues that lack a `P*` priority label. The canonical label vocabulary and rubric live in [`docs/agent/triage-vocabulary.md`](../../../docs/agent/triage-vocabulary.md) — read it before starting.

## Steps

1. **Fetch state in parallel.** Call `mcp__github__list_issues` (state OPEN) and `mcp__github__list_pull_requests` (state open) for `dmooney/parish`. Both responses are large — save to disk and use `jq` rather than reading raw output.

2. **Find the un-PR'd set.** Extract every `#NNN` reference from PR titles + bodies, intersect with open issue numbers. Issues NOT referenced by any open PR are candidates.

3. **Filter to un-triaged.** Keep only issues whose existing labels do not contain any of `P0`, `P1`, `P2`, `P3`. Don't relabel issues that already have a priority unless the user asks for a re-triage.

4. **Classify.** For each remaining issue, read title + body and assign:
   - **Exactly one priority** (`P0`/`P1`/`P2`/`P3`) using the rubric in `triage-vocabulary.md`.
   - **At least one theme** label. Multiple is fine when an issue genuinely spans themes (e.g. `security` + `infra` for a workflow vuln).
   - When uncertain between two priorities, pick the lower-urgency one and let a human escalate.

5. **Apply.** Call `mcp__github__issue_write` with `method: "update"` and the **union** of (existing labels) + (theme labels) + (priority). `issue_write` *replaces* the set, so always include pre-existing labels like `bug`, `security`, `ready-for-test`. Dispatch in parallel batches of ~15.

6. **Verify.** Re-list with `labels: ["P0"]`, `["P1"]`, `["P2"]`, `["P3"]` and confirm counts match what you applied. Random-sample a few issues with `issue_read` (`get_labels`) to confirm theme labels stuck.

7. **Report.** Summarize counts by priority and theme. Link to GitHub filter URLs (`https://github.com/dmooney/Parish/issues?q=is%3Aopen+label%3AP0` etc.). Flag any issue carrying `ready-for-test` without an open PR — those usually need closing, not implementation.

## Notes

- New labels added to `triage-vocabulary.md` are auto-created on first use by `issue_write`, but ship without colors/descriptions. After this skill creates one, set its color in the GitHub UI.
- If a new theme is needed that isn't in the vocabulary, **stop and ask the user** before inventing a label. Update `triage-vocabulary.md` first.
- The `triage-audit` workflow runs weekly and posts a summary listing un-triaged issues — that's the usual trigger for invoking this skill.
