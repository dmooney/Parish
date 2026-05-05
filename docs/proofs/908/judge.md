Verdict: sufficient
Technical debt: clear

PR #908 correctly extracts language configuration from the engine into the mod
manifest layer. The engine (`parish-npc`) no longer hardcodes any reference to
`Irish`, `irish_words`, or Hiberno-English idioms in its prompt builders; that
flavour now lives exclusively in `mods/rundale/mod.toml` and the Rundale prompt
templates, which is the correct ownership boundary — the engine is generic, the
mod is opinionated. The `LanguageSettings` struct and `language_directive()`
function are well-scoped: they accept any BCP 47 locale pair and produce a
directive that handles the en-US spelling-discipline case, the code-switching
case, and the monolingual case as three distinct branches.

The serde `#[serde(alias = "irish_words")]` attribute on `language_hints` in
both `Tier1LlmResponse` and `Tier2LlmResponse` preserves backward
compatibility with existing saves and LLM responses that used the old key.
Test coverage is appropriate: 4 directive-matrix tests cover the en-IE/ga-IE,
en-US/None, and fr-FR/None cases plus the tier1 prompt embedding assertion; 4
`SettingConfig` serde tests cover round-trip with both fields, default
omission, partial omission, and the `GameMod` accessor methods. Architecture
fitness and wiring parity gates pass. No unexplained `#[allow]` annotations.
No placeholder debt markers.
