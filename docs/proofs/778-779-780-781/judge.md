Verdict: sufficient
Technical debt: clear

This PR fixes four test-quality issues — no production logic was changed except
extracting `syncFocailOnViewportChange` from an inline component handler into an
exported function (which the component immediately calls, preserving behavior).

#778: Both assertions now derive their expected values rather than comparing
against literals. The world_graph test reads world.json and verifies the loader
parses every entry; the headless test verifies every visited destination is a
real graph location and that at least 14 were visited.

#779: The 50-seed "find any propagation" loops are replaced with 200-seed rate
assertions (50-70%). This directly tests the production transmission probability
and will fail if it is silently regressed. Structural invariants are also verified
on the first successful transmission.

#780: The weather seasonal bias test is now driven through tick() over 600
simulated hours, exercising the same hourly-gate and min-duration logic as
production rather than bypassing them via field mutation and compute_transition().

#781: The regression test now calls syncFocailOnViewportChange (the extracted
handler logic) rather than focailOpen.set(false) directly. The test would fail
if the reset logic were deleted or inverted.

All 840 Rust tests pass; all 304 UI unit tests pass. No debt markers present.
