# Proof Evidence — Issue #697: NpcManager decomposition

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: claude/cranky-tharp-04ca00

## Requirement

`parish-npc` crates were restructured: `tick_schedules`, `assign_tiers`,
`apply_tier4_events`, and `tick_banshee` extracted from `manager.rs` into
dedicated modules (`schedule`, `tier_assign`, `tier4`, `banshee`). The game
must start, move the player, tick NPC schedules, and emit valid JSON for all
harness commands.

## Unit and Integration Tests

Command:

```sh
cargo test -p parish-npc
```

Result:

```
cargo test: 392 passed (4 suites, 0.08s)
```

All 392 tests pass. Architecture fitness gate (`architecture_fitness.rs`)
enforces no orphaned source files and no runtime-dep leakage.

## Harness Walkthrough

Command:

```sh
cargo run -p parish -- --script parish/testing/fixtures/test_walkthrough.txt
```

Result:

```json
{"command":"look","result":"looked","description":"The small village of Kilteevan — a handful of whitewashed cottages clustered around a well and an old stone bridge over a shallow stream. Smoke drifts from chimneys. A rooster crows from behind a low wall. The clear sky hangs over the quiet street. It is morning.","location":"Kilteevan Village","time":"Morning","season":"Spring","new_log_lines":["The small village of Kilteevan — a handful of whitewashed cottages clustered around a well and an old stone bridge over a shallow stream. Smoke drifts from chimneys. A rooster crows from behind a low wall. The clear sky hangs over the quiet street. It is morning.","You can go to: The Crossroads (13 min on foot), The Forge (1 min on foot), The Holy Well (1 min on foot), The Mill (1 min on foot), The Weaver's Cottage (2 min on foot), Knockcroghery Village (85 min on foot), St. Brigid's Church (9 min on foot), Murphy's Farm (21 min on foot), The Lime Kiln (1 min on foot), The Letter Office (11 min on foot)","A young woman heads off down the road.","A lean, red-haired young man with hard eyes heads off down the road.","An older man heads off down the road.","An older woman with sharp eyes and herb-stained fingers heads off down the road."]}
{"command":"go to the crossroads","result":"moved","to":"The Crossroads","minutes":13,"narration":"You walk along the road north past low fields to the crossroads. (13 minutes on foot)","location":"The Crossroads","time":"Morning","season":"Spring","new_log_lines":["You walk along the road north past low fields to the crossroads. (13 minutes on foot)","","A quiet crossroads where four narrow roads meet. A weathered stone wall lines the eastern side, half-hidden by brambles. The clear sky stretches over the flat midlands. It is morning.\nYou can go to: Darcy's Pub (1 min on foot), St. Brigid's Church (11 min on foot), The Letter Office (4 min on foot), The Hedge School (2 min on foot), Connolly's Shop (1 min on foot), Kilteevan Village (13 min on foot), The Hurling Green (4 min on foot)"]}
{"command":"/status","result":"system_command","response":"Location: The Crossroads | Morning | Spring","location":"The Crossroads","time":"Morning","season":"Spring","new_log_lines":["Location: The Crossroads | Morning | Spring"]}
{"command":"/map","result":"system_command","response":"No tile sources configured.","location":"The Crossroads","time":"Morning","season":"Spring","new_log_lines":["No tile sources configured."]}
{"command":"/help","result":"system_command","response":"Available commands:\n  /about ...","location":"The Crossroads","time":"Morning","season":"Spring","new_log_lines":[]}
```

Exit code: 0.

NPC schedule ticks fired correctly on startup (NPCs departed from Kilteevan
Village in the log lines), confirming `schedule::tick_schedules` is wired
correctly through the `NpcManager` wrapper.
