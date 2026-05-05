# Evidence: replace serde_yml with serde_yaml; suppress rsa Marvin Attack advisory

Evidence type: gameplay transcript
Date: 2026-05-04
Branch: claude/fix-ci-main-1erit

## Problem

CI on main turned red after PR #891 merged: `cargo audit` (triggered by the
`Cargo.lock` change that added `lru`) found two advisories treated as errors:

- RUSTSEC-2025-0068 — `serde_yml 0.0.12` unsound and unmaintained
- RUSTSEC-2023-0071 — `rsa 0.9.10` Marvin Attack timing sidechannel (no fix available)

## Fix

1. Replaced `serde_yml` with `serde_yaml 0.9` (sound, no RUSTSEC advisory).
   Single call site: `serde_yml::from_str` → `serde_yaml::from_str` in
   `parish-core/src/prompts/mod.rs`.

2. Added `parish/.cargo/audit.toml` to suppress RUSTSEC-2023-0071. Our
   `jsonwebtoken` usage is RSA signature *verification* with a public key only.
   The Marvin Attack requires an RSA *decryption* oracle; we never decrypt.

## cargo audit — after fix

```
$ cd parish && cargo audit --no-fetch
    Loaded 1067 security advisories (from /root/.cargo/advisory-db)
warning: 18 allowed warnings found
```

Exit code 0. No errors. The 18 allowed warnings are all unmaintained GTK/Tauri
bindings (pre-existing; not introduced by this PR).

## parish-core tests — after serde_yaml swap

```
$ cargo test -p parish-core
test prompts::tests::parses_minimal_file ... ok
test prompts::tests::substitutes_known_variables ... ok
test prompts::tests::leaves_unknown_variables_unchanged ... ok
test prompts::tests::substitutes_multiple_occurrences ... ok
test prompts::tests::does_not_rescan_substituted_values ... ok
test prompts::tests::render_system_only_returns_system_messages ... ok
test prompts::tests::render_system_joins_multiple_system_messages_with_blank_line ... ok
test prompts::tests::substitution_handles_braces_in_template_safely ... ok
test prompts::tests::parse_panics_on_malformed_yaml ... ok
test prompts::tests::parse_panics_on_missing_messages_field ... ok

test result: ok. 293 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## Fixture smoke test

```
$ cargo run -p parish -- --script testing/fixtures/test_speed_assertions.txt
{"command":"/status","result":"system_command","response":"Location: Kilteevan Village | Morning | Spring",...}
{"command":"/speed slow","result":"system_command","response":"The parish slows to a gentle amble.",...}
{"command":"/speed normal","result":"system_command","response":"The parish settles into its natural stride.",...}
{"command":"/quit","result":"quit","location":"Kilteevan Village",...}
```

All fixture commands produced expected output. No regressions from the YAML
parser swap.
