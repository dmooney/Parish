# Proof Evidence — PR #908: mod language settings (player_language + native_language)

Evidence type: gameplay transcript
Date: 2026-05-05
Branch: claude/add-mod-language-settings-O07Gn

## Requirement

The engine previously hardcoded `Pepper your speech with Irish` and an
`irish_words` metadata key directly inside `parish-npc` prompt builders.
This PR lifts the language configuration out of the engine and into the mod
manifest (`mods/rundale/mod.toml`), so any mod can declare a `player_language`
(BCP 47, defaults `"en"`) and an optional `native_language`. The engine then
generates a `LANGUAGE:` directive and injects it into every dialogue system
prompt (tier1, tier2, tier3, reactions). A serde `alias = "irish_words"`
preserves backward compatibility with existing saves and LLM responses.

## cargo test -p parish-npc language_directive

Command:

```sh
cargo test -p parish-npc language_directive
```

Result — all 4 new directive tests pass:

```
running 4 tests
test tests::language_directive_fr_fr_no_native ... ok
test tests::language_directive_en_ie_with_native_ga_ie ... ok
test tests::language_directive_en_us_no_native ... ok
test tests::tier1_prompt_contains_language_directive ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 380 filtered out; finished in 0.00s
```

The four tests exercise the matrix:

- **en-IE / ga-IE** — directive must contain `"en-IE"`, warn against `"en-US"`
  spellings, name `"ga-IE"`, mention `"language_hints"`, and must NOT include the
  mono-language restriction.
- **en-US / None** — must contain `"en-US"`, must NOT warn against en-US
  spellings, must include the mono-language restriction.
- **fr-FR / None** — must contain `"fr-FR"`, must NOT mention `"en-US spellings"`,
  must include the mono-language restriction.
- **tier1 prompt embedding** — `build_tier1_system_prompt` for an en-IE/ga-IE
  `LanguageSettings` must embed `"LANGUAGE:"`, `"en-IE"`, and `"ga-IE"`.

### Hand-traced directive for en-IE / ga-IE

`language_directive(&LanguageSettings::new("en-IE", Some("ga-IE".into())))` produces
(from `parish/crates/parish-npc/src/lib.rs:342–371`):

```
LANGUAGE: Speak in en-IE. Use spelling, idioms, and conventions appropriate to
that BCP 47 locale. Never use en-US spellings such as "color", "realize",
"favor", "neighbor", or "-ize" verb endings — use the spelling appropriate to
en-IE. Where a native speaker would naturally code-switch, sprinkle words and
short phrases from ga-IE into your dialogue and record them in the
`language_hints` metadata array.
```

## cargo test -p parish-core game_mod

Command:

```sh
cargo test -p parish-core game_mod
```

Result — all 33 `game_mod` tests pass, including the 4 new `SettingConfig`
language-field tests:

```
running 33 tests
test game_mod::tests::discover_mods_treats_missing_kind_as_setting ... ok
test game_mod::tests::discover_mods_rejects_two_settings ... ok
test game_mod::tests::discover_mods_finds_setting_and_auxiliary_in_lex_order ... ok
test game_mod::tests::setting_config_with_both_languages ... ok
test game_mod::tests::discover_mods_requires_a_setting ... ok
test game_mod::tests::setting_config_defaults_player_language_to_en_when_omitted ... ok
test game_mod::tests::test_anachronism_entry_deserialize ... ok
test game_mod::tests::test_anachronism_entry_deserialize_legacy_reason ... ok
test game_mod::tests::setting_config_with_only_player_language ... ok
test game_mod::tests::game_mod_accessors_expose_language_settings ... ok
test game_mod::tests::test_check_festival ... ok
test game_mod::tests::test_encounter_text_lookup ... ok
test game_mod::tests::test_festival_def_deserialize ... ok
test game_mod::tests::test_interpolate_template ... ok
test game_mod::tests::test_interpolate_template_empty ... ok
test game_mod::tests::test_interpolate_template_no_placeholders ... ok
test game_mod::tests::test_interpolate_template_missing_key ... ok
test game_mod::tests::test_load_nonexistent_dir ... ok
test game_mod::tests::test_load_mod_with_pronunciations ... ok
test game_mod::tests::test_loading_config_deserialize ... ok
test game_mod::tests::test_load_rejects_directory_traversal_in_manifest ... ok
test game_mod::tests::test_mod_npcs_path ... ok
test game_mod::tests::test_mod_world_path ... ok
test game_mod::tests::test_load_mod_from_directory ... ok
test game_mod::tests::test_name_hints_case_insensitive ... ok
test game_mod::tests::test_pronunciation_entry_matches_via_word_fallback ... ok
test game_mod::tests::test_load_real_default_mod ... ok
test game_mod::tests::test_pronunciation_entry_deserialize ... ok
test game_mod::tests::test_ui_config_custom ... ok
test game_mod::tests::test_name_hints_for_matching ... ok
test game_mod::tests::test_ui_config_defaults ... ok
test game_mod::tests::test_ui_config_legacy_default_accent ... ok
test game_mod::tests::test_real_mod_npc_name_hints ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 279 filtered out; finished in 0.02s
```

The 4 new deserialisation tests cover: both fields present, `player_language`
defaults to `"en"` when absent, `native_language` absent, and the `GameMod`
accessor methods that surface both fields.

## cargo test (engine crates, excluding GTK-dependent Tauri)

Command (full workspace minus Tauri/GTK which requires system libraries
unavailable in this environment):

```sh
cargo test -p parish-npc -p parish-core -p parish-inference -p parish-world \
           -p parish-config -p parish-types -p parish-persistence \
           -p parish-palette -p parish-input -p parish-server -p parish
```

Result summary (all suites):

```
test result: ok. 81 passed; 0 failed; 0 ignored
test result: ok. 312 passed; 0 failed; 1 ignored   (parish-npc unit tests)
test result: ok. 3 passed; 0 failed; 0 ignored      (parish-npc gossip integration)
test result: ok. 6 passed; 0 failed; 0 ignored      (parish-npc tier2 LLM integration)
test result: ok. 14 passed; 0 failed; 0 ignored
test result: ok. 6 passed; 0 failed; 0 ignored
test result: ok. 223 passed; 0 failed; 7 ignored    (parish-core unit tests)
test result: ok. 31 passed; 0 failed; 0 ignored
test result: ok. 119 passed; 0 failed; 0 ignored
test result: ok. 6 passed; 0 failed; 0 ignored
test result: ok. 384 passed; 0 failed; 0 ignored
test result: ok. 3 passed; 0 failed; 0 ignored
test result: ok. 6 passed; 0 failed; 0 ignored
test result: ok. 18 passed; 0 failed; 0 ignored
test result: ok. 105 passed; 0 failed; 0 ignored
test result: ok. 88 passed; 0 failed; 0 ignored
test result: ok. 147 passed; 0 failed; 0 ignored
test result: ok. 12 passed; 0 failed; 0 ignored    (parish-server)
test result: ok. 28 passed; 0 failed; 0 ignored    (parish / headless CLI)
```

No failures across any engine crate.

## Rendered tier1 prompt sample

`build_tier1_system_prompt` appends `language_directive(language)` at the end
of the prompt body (see `parish/crates/parish-npc/src/lib.rs:463`). For
`LanguageSettings { player: "en-IE", native: Some("ga-IE") }` the appended
block is:

```
LANGUAGE: Speak in en-IE. Use spelling, idioms, and conventions appropriate to
that BCP 47 locale. Never use en-US spellings such as "color", "realize",
"favor", "neighbor", or "-ize" verb endings — use the spelling appropriate to
en-IE. Where a native speaker would naturally code-switch, sprinkle words and
short phrases from ga-IE into your dialogue and record them in the
`language_hints` metadata array.
```

The `tier1_prompt_contains_language_directive` test (line 1254) asserts that
`"LANGUAGE:"`, `"en-IE"`, and `"ga-IE"` all appear in the returned string.

## Backward compatibility note

`parish/crates/parish-npc/src/lib.rs:230` and `:250` carry:

```rust
#[serde(default, alias = "irish_words")]
pub language_hints: Vec<LanguageHint>,
```

on both the `Tier1LlmResponse` and `Tier2LlmResponse` structs. Any saved game
or LLM response that used the old `irish_words` JSON key is still parsed
correctly; the alias is transparent to consumers. The `mods/rundale/prompts/
tier1_system.txt` no longer contains the hardcoded `Pepper your speech with Irish`
sentence or the `irish_words` key — that content is now driven entirely by
`language_directive()` from the mod manifest's `player_language`/`native_language`
settings.
