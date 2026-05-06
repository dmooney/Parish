Evidence type: gameplay transcript

## PR #909 — Technical debt inventory for all 14 crates and frontend

### Verification

All 15 `TODO.md` files created, one per crate plus frontend:

```
$ find parish/crates parish/apps/ui -name TODO.md | sort
parish/apps/ui/TODO.md
parish/crates/parish-cli/TODO.md
parish/crates/parish-config/TODO.md
parish/crates/parish-core/TODO.md
parish/crates/parish-geo-tool/TODO.md
parish/crates/parish-inference/TODO.md
parish/crates/parish-input/TODO.md
parish/crates/parish-npc/TODO.md
parish/crates/parish-npc-tool/TODO.md
parish/crates/parish-palette/TODO.md
parish/crates/parish-persistence/TODO.md
parish/crates/parish-server/TODO.md
parish/crates/parish-tauri/TODO.md
parish/crates/parish-types/TODO.md
parish/crates/parish-world/TODO.md
```

### Format verification

Every file follows the techdebt skill format with `## Open` / `## In Progress` / `## Done` sections, TD-XXX IDs, severity column, and concrete file:line locations:

```
$ grep -l '## Open' parish/crates/*/TODO.md parish/apps/ui/TODO.md | wc -l
      15
```

### Item counts

| Area | Items |
|---|---|
| parish/apps/ui | 30 |
| parish-cli | 19 |
| parish-config | 8 |
| parish-core | 14 |
| parish-geo-tool | 11 |
| parish-inference | 23 |
| parish-input | 8 |
| parish-npc | 15 |
| parish-npc-tool | 15 |
| parish-palette | 7 |
| parish-persistence | 14 |
| parish-server | 20 |
| parish-tauri | 5 |
| parish-types | 7 |
| parish-world | 11 |

**Total: 207 items** (0 P0, 21 P1, 126 P2, 59 P3)

### Sample entry (parish-npc/TODO.md)

```
| TD-001 | Weak Tests | P1 | src/banshee.rs:1-466 | Death system has no test for multiple
simultaneous dooms, exact DOOM_HERALD_WINDOW_HOURS boundary (12h), or clock-rewind
scenarios. This is critical game logic. |
```

### Quality gate

```
$ just check  # fmt, clippy, tests — all pass
```

No Rust code was changed — all 15 files are new markdown documentation files only.
