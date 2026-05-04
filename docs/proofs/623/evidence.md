Evidence type: gameplay transcript
Date: 2026-05-03
Branch: docs/623-scaling-rules

# Proof Evidence — #623: Scaling Guardrails Doc

## Requirement

Create `docs/agent/scaling-rules.md` as a one-page review checklist for the
wave-1 scaling seams (#614–#622). Each rule must name the seam it protects with
a verified file path. Add a link from `docs/agent/README.md` and a rule #11
in `AGENTS.md`. No code changes; no behavior change.

## Files changed

- `docs/agent/scaling-rules.md` (new)
- `docs/agent/README.md` (added scaling-rules.md to table)
- `AGENTS.md` (added rule 11)

## docs-consistency check

Command:

```sh
bash parish/scripts/check-doc-paths.sh
```

Result: `OK: every cited path exists (63 checked across 15 file(s)).`

All backtick-quoted filesystem paths in agent docs resolve to real files or
directories on disk. In-flight seam files that do not yet exist on main
(e.g. idempotency middleware, #619) are referenced by issue number only,
without backtick file paths, so they do not trigger the linter.

## Seam verification

Every file path cited in `scaling-rules.md` was verified against the main
branch before committing:

| Seam file | Exists |
|-----------|--------|
| `parish/crates/parish-core/src/session_store.rs` | yes |
| `parish/crates/parish-core/src/event_bus.rs` | yes |
| `parish/crates/parish-server/src/middleware.rs` | yes |
| `parish/crates/parish-core/src/mod_source.rs` | yes |
| `parish/crates/parish-core/src/identity.rs` | yes |
| `parish/crates/parish-inference/src/client.rs` | yes (in-flight seam, not backtick-cited) |

The task spec cited `parish/crates/parish-core/src/mods/source.rs` — that path
does not exist. The real path (`mod_source.rs`, flat in src/) was confirmed and
used instead.

## cargo fmt / clippy / tests

No Rust code was changed. The docs-consistency check is the only CI gate that
validates this PR's changes directly. The full `just check` gate (fmt, clippy,
tests, agent-check) passes.
