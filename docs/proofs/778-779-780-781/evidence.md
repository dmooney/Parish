Evidence type: gameplay transcript
Date: 2026-05-04
Branch: fix/778-779-780-781-test-quality

# Proof Evidence — Fixes #778, #779, #780, #781: Test Quality

## Summary

Four test-quality issues fixed. In each case the production code path is
unchanged; only the tests were deficient.

## #778 — Exact-count assertions replaced with derived/range checks

### world_graph_integration.rs: test_parish_json_location_count

`assert_eq!(graph.location_count(), 22)` replaced with a derived assertion:
the test now reads `mods/rundale/world.json` directly, counts the entries in
the `"locations"` array, and asserts `graph.location_count() == json_count`.
This will catch any future loader regression that silently drops locations,
not just a mismatch against a hardcoded literal.

### headless_script_tests.rs: test_all_locations_reachable

The hardcoded list of 14 location names replaced with:
1. Every resolved destination must appear in the world graph (verifies no
   phantom locations).
2. At least 14 distinct locations were visited (`>= 14`), so the test
   survives graph growth without requiring a fixture update.

Command run:

```sh
cargo test -p parish --test world_graph_integration
```

Result: 28 passed

```sh
cargo test -p parish --test headless_script_tests
```

Result: 74 passed

## #779 — Gossip retry loops replaced with rate assertions

The three tests in `gossip_integration.rs` that used "find any of 50 seeds
that propagates" loops were replaced with:

- A loop over 200 deterministic seeds that asserts the transmission rate is
  between 50% and 70% (the documented ~60%). This directly catches a
  regression in transmission probability; the previous code would pass even
  if the rate dropped to 6%.
- Structural invariants (source recorded, listener in known_by) are verified
  on the first seed that does transmit, so both correctness and probability
  are tested.

Command run:

```sh
cargo test -p parish-npc --test gossip_integration
```

Result: 3 passed

## #780 — Weather seasonal bias test driven through tick()

`test_weather_seasonal_bias` previously bypassed `tick()` by resetting the
private `last_check_hour` field and calling `compute_transition()` directly.
The test now drives the engine through `tick()` over 600 simulated game-hours
(1 hour per iteration), which exercises the same hourly-gate and
min-duration logic that production uses.

With 600 hours and a 2-hour min-duration, roughly 150-300 transition checks
occur — enough for the winter vs summer rain signal to be clearly observable.
The assertion (`winter_rain > summer_rain` with seed 12345) still holds.

Command run:

```sh
cargo test -p parish-world test_weather_seasonal_bias
```

Result: 1 passed

## #781 — focailOpen test exercises the actual handler

The `syncFocailOnViewportChange(matches: boolean)` function was extracted
from the inline `onChange` handler in `+page.svelte` into `game.ts` as an
exported function. The handler in `+page.svelte` now delegates to it.

The test now imports and exercises `syncFocailOnViewportChange` directly:
- `matches=false` (mobile-to-desktop transition) must reset `focailOpen`.
- `matches=true` (desktop-to-mobile or still mobile) must leave it unchanged.

Previously the test just called `focailOpen.set(false)` directly, which
would pass even if the entire handler were deleted.

Command run:

```sh
just ui-test
```

Result: 304 passed (23 suites)

## Full Rust suite

```sh
cargo test -p parish -p parish-npc -p parish-world
```

Result: 840 passed (14 suites)
