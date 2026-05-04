Verdict: sufficient
Technical debt: clear

PR #623 adds `docs/agent/scaling-rules.md` — a one-page checklist of the nine
scaling guardrail rules introduced in wave-1 (#614–#622). No code changes;
documentation only.

Every backtick-quoted file path in the new doc resolves to a real file on main.
The docs-consistency CI check (`check-doc-paths.sh`) passes. In-flight seams
(#617, #618, #619) are cited by issue number without backtick file paths so they
do not trigger the linter. The corrected path for the ModSource seam
(`mod_source.rs`, not `mods/source.rs`) was confirmed against main before
committing. Links from `docs/agent/README.md` and `AGENTS.md` rule #11 are in
place.
