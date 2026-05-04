# Proof Evidence — PR #883: remove duplicate reqwest dependency

Evidence type: gameplay transcript
Date: 2026-05-04
Branch: fix/parish-core-duplicate-reqwest-dep

## Requirement

`parish-core/Cargo.toml` had `reqwest = { workspace = true }` listed twice
(lines 19 and 30). The duplicate key caused `cargo metadata` to exit with
`error: duplicate key`, breaking `just run` on main. The fix removes the
second entry.

## cargo metadata

Command:

```sh
cargo metadata --no-deps -q | python3 -c \
  "import sys,json; d=json.load(sys.stdin); pkgs=[p['name'] for p in d['packages']]; \
   print('packages:', len(pkgs)); print('parish-core in workspace:', 'parish-core' in pkgs)"
```

Result:

```
packages: 14
parish-core in workspace: True
```

Exit code: 0. Workspace resolves cleanly with no duplicate-key error.

## parish-core tests

Command:

```sh
cargo test -p parish-core
```

Result:

```
cargo test: 309 passed, 4 ignored (6 suites, 5.33s)
```

All 309 tests pass including architecture fitness and wiring parity gates.
No regressions introduced by removing the duplicate key.
