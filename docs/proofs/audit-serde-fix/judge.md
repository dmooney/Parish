# Judge Verdict: audit-serde-fix

Verdict: sufficient
Technical debt: clear

## Review

The evidence demonstrates:

1. `cargo audit` exits 0 with no errors after the fix (18 pre-existing unmaintained
   warnings for GTK/Tauri crates, not introduced here).

2. All `parish-core` tests pass after the `serde_yml` → `serde_yaml` swap, including
   all `prompts::tests::*` that directly exercise the changed call site.

3. A gameplay fixture (`test_speed_assertions.txt`) runs clean, confirming no
   runtime regression from the YAML parser swap.

4. The `rsa` advisory ignore in `.cargo/audit.toml` is correctly scoped: the
   Marvin Attack (RUSTSEC-2023-0071) targets RSA decryption, not RSA signature
   verification. The codebase uses `jsonwebtoken` for Cloudflare Access JWT
   *verification* only (public key, no decryption). The justification is accurate
   and the ignore is the standard practice when no upstream fix is available.

No placeholder debt, no unexplained suppressions, no behaviour change.
