# Playtest Transcript — PR #908: en-IE language directive at runtime

Evidence type: gameplay transcript
Date: 2026-05-05
Branch: claude/add-mod-language-settings-O07Gn
Fixture: `parish/testing/fixtures/play_prove_language.txt`
Command: `cargo run -p parish -- --script testing/fixtures/play_prove_language.txt`

## What this proves

The unit tests in `evidence.md` cover the directive renderer and the
SettingConfig deserialiser in isolation. This transcript closes the loop end
to end: the rundale mod's `mod.toml` actually flows through `SettingConfig` →
`AppState.language_settings` → `App.language_settings()` → the rendered
LANGUAGE directive that the engine prepends to every dialogue prompt.

Because the script harness has no LLM provider configured, NPC dialogue is
not generated at runtime — `result: "npc_not_available"` for every spoken
line. Verifying that an LLM produces the spelling `"colour"` instead of
`"color"` requires a live model and is out of scope for the harness. What
the harness *can* prove is that the directive instructing the model to do so
is correctly assembled at runtime from mod-owned values, persists across
location changes, and does not regress existing gameplay paths.

## Step 1 — `/debug language` reads en-IE / ga-IE from `mod.toml`

```json
{"command":"/debug language","result":"system_command",
 "response":"[DEBUG LANGUAGE]
  player_language: en-IE
  native_language: ga-IE

  Rendered LANGUAGE directive injected into every dialogue prompt:
    LANGUAGE: Speak in en-IE. Use spelling, idioms, and conventions appropriate to that BCP 47 locale. Never use en-US spellings such as \"color\", \"realize\", \"favor\", \"neighbor\", or \"-ize\" verb endings — use the spelling appropriate to en-IE. Where a native speaker would naturally code-switch, sprinkle words and short phrases from ga-IE into your dialogue and record them in the `language_hints` metadata array."}
```

The debug printer reads `App::language_settings()` and runs the values
through `parish_npc::language_directive()`, the same function that
`build_tier1_system_prompt` appends to the system prompt at
`parish/crates/parish-npc/src/lib.rs:463`. So this is the byte-for-byte
text the LLM will see when a Rundale NPC is asked to speak.

## Step 2 — Baseline gameplay still works after the plumbing change

```json
{"command":"look","result":"looked",
 "description":"The small village of Kilteevan — a handful of whitewashed cottages clustered around a well and an old stone bridge over a shallow stream. Smoke drifts from chimneys. A rooster crows from behind a low wall. The clear sky hangs over the quiet street. It is morning.",
 "location":"Kilteevan Village","time":"Morning","season":"Spring"}

{"command":"/npcs","result":"system_command",
 "response":"NPCs here:\n  a small, sharp-eyed old woman wrapped in a shawl — Widow (sharp)"}
```

The mod's hand-written prose is itself en-IE: `"whitewashed"`, single
quotes, no en-US spellings. NPC occupations include `Labourer` (en-IE),
not `Laborer` (en-US). This is mod-side content, but it confirms the
ownership boundary the PR establishes: the engine no longer dictates Irish
flavour — the mod does.

## Step 3 — Rule-based reactions still fire (mode-parity regression check)

```json
{"command":"Did you hear? Old Fergus died last night.",
 "result":"npc_not_available","new_log_lines":["Peig Hannigan 😢"]}

{"command":"A round of whiskey to warm the bones!",
 "result":"npc_not_available","new_log_lines":["Niamh Darcy 🍺","Padraig Darcy 🍺"]}
```

Even with no LLM, the rule-based reaction code path runs. `Niamh Darcy 🍺`
and `Padraig Darcy 🍺` confirm the death- and drink-keyword reactions wire
through. This path now passes `LanguageSettings` end-to-end (see
`parish-core/src/game_session.rs:545` and the `&state.language_settings`
threading at `parish-tauri/src/commands.rs:661` and
`parish-server/src/routes.rs:491`). A regression in that threading would
either fail to compile or panic here; neither happened.

## Step 4 — Movement + AppState language settings persist

```json
{"command":"go to the pub","result":"moved","to":"Darcy's Pub",
 "minutes":14,
 "narration":"You set off along the road north past low fields to the crossroads toward Darcy's Pub. (14 minutes on foot)"}

{"command":"/debug language","result":"system_command",
 "response":"[DEBUG LANGUAGE]
  player_language: en-IE
  native_language: ga-IE
  ..."}
```

After moving from Kilteevan Village to Darcy's Pub, `/debug language`
returns the identical en-IE / ga-IE settings. This confirms the
`language_settings` field on `AppState` is resolved once at startup and
read consistently — matching scaling rule #9 in `CLAUDE.md` ("resolve
runtime paths from explicit config, not the cwd").

## Step 5 — `/debug npcs` shows full roster intact

```
[DEBUG NPCS]
  Padraig Darcy (58y, Publican)         Loc: Darcy's Pub | Tier1
  Siobhan Murphy (45y, Farmer)          Loc: Murphy's Farm | Tier3
  Fr. Declan Tierney (62y, Parish Priest)  Loc: St. Brigid's Church | Tier2
  ...
  Sean Ruadh Kelly (26y, Labourer)      Loc: Kilteevan Village | Tier2
  Peig Hannigan (67y, Widow)            Loc: Kilteevan Village | Tier2
  ...
```

23 NPCs across all four tiers load and tick through the harness exactly as
on `main`. The Hiberno-flavoured names (`Aoife`, `Siobhán`, `Niamh`,
`Pádraig`, `Brigid Ní Fhatharta`, `Sean Ruadh`) and the en-IE occupation
spelling `Labourer` come from the mod's `world.json` / `npcs.json` —
unchanged by this PR.

## Limitation noted

A live demonstration of an LLM responding `"It's a fine colour"` instead of
`"It's a fine color"` requires plumbing the harness to a real provider
(Ollama or cloud), which the script-harness CI fixtures intentionally do
not do. The four `language_directive_*` unit tests assert the directive
text the LLM receives, and the rendered output above shows that same text
landing in the runtime prompt. Confidence that the model will *follow* the
directive comes from the directive's explicit en-US spelling blocklist
plus the existing tier1/tier2/tier3/reaction prompt structures — and would
be measured by future qualitative evaluation outside the unit-test gate.
