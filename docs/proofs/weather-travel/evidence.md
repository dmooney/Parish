# Weather-Travel Proof Evidence

Evidence type: gameplay transcript
Date: 2026-05-03
Branch: claude/zealous-faraday-3xNnd
PR: #527

## Feature

Weather-impeded travel: multiplier table slows journeys during rain, fog, and storm; Storm has a ~35% abort chance. Feature flag `weather-travel` (default-on) controls the system. `/weather <kind>` harness command forces weather state for testing.

## Unit Tests

```
cargo test -p parish-world
running 155 tests
...
test weather_travel::tests::apply_multiplier_rounds_up ... ok
test weather_travel::tests::apply_multiplier_saturates_near_u16_max ... ok
test weather_travel::tests::all_weather_values_have_a_finite_multiplier ... ok
test weather_travel::tests::clear_helper_matches_compute_result ... ok
test weather_travel::tests::clear_weather_is_silent_and_unchanged ... ok
test weather_travel::tests::fog_is_slower_than_heavy_rain ... ok
test weather_travel::tests::heavy_rain_turns_the_boreen_to_a_creek ... ok
test weather_travel::tests::light_rain_in_spring_is_grand_for_the_land ... ok
test weather_travel::tests::light_rain_in_winter_speaks_of_the_cold_soak ... ok
test weather_travel::tests::only_storm_can_abort_the_journey ... ok
test weather_travel::tests::pick_roll_selects_different_lines_from_the_pool ... ok
test weather_travel::tests::storm_below_threshold_forces_the_player_back ... ok
test weather_travel::tests::storm_doubles_travel_time ... ok
test result: ok. 155 passed; 0 failed
```

## Live CLI Play-Test

```
$ rm -f mods/rundale/parish-flags.json && rm -rf saves
$ cat <<'EOF' | parish --provider simulator --game-mod mods/rundale
/status
go crossroads
go kilteevan village

/weather Light Rain
go crossroads
go kilteevan village

/weather Heavy Rain
go crossroads
go kilteevan village

/weather Fog
go crossroads
go kilteevan village

/weather Storm
go crossroads
go kilteevan village
go crossroads
go kilteevan village
go crossroads
go kilteevan village

/flag disable weather-travel
go crossroads
go kilteevan village
/quit
EOF
```

### Baseline — Clear morning

```
> Location: Kilteevan Village | Morning | Spring

> You walk along the road north past low fields to the crossroads.
  (13 minutes on foot)

> You walk along the Kilteevan road heading south past low fields.
  (13 minutes on foot)
```

Silent, unchanged. Feature does not speak up for fair weather.

### Light rain — spring voicing

```
> Weather forced to Light Rain.

A soft rain falls — the sort the country calls 'grand for the land' —
and you pull your shawl tighter and walk on.
You walk along the road north past low fields to the crossroads.
(13 minutes on foot) (slowed by the weather from 13 to 15 minutes)
```

### Heavy rain — the boreen is a creek

```
> Weather forced to Heavy Rain.

The rain comes down in sheets; the ruts in the road are running brown.
You walk along the road north past low fields to the crossroads.
(13 minutes on foot) (slowed by the weather from 13 to 19 minutes)

The boreen is a creek and your boots a pair of buckets by the time
you've gone a furlong.
You walk along the Kilteevan road heading south past low fields.
(13 minutes on foot) (slowed by the weather from 13 to 19 minutes)
```

Different line on each traversal — pool broad enough that walking back reads differently.

### Fog — slower than heavy rain

```
> Weather forced to Fog.

You can hear your own boots on the road but not the cattle in the
field, so thick is the fog this morning.
You walk along the road north past low fields to the crossroads.
(13 minutes on foot) (slowed by the weather from 13 to 20 minutes)

The fog turns every gatepost into a stranger; you slow down to be sure.
You walk along the Kilteevan road heading south past low fields.
(13 minutes on foot) (slowed by the weather from 13 to 20 minutes)
```

Fog beats heavy rain by a minute (20 vs 19) — visibility, not wetness, slows the walker.

### Storm — doubled time and abort

```
> Weather forced to Storm.

A gust takes your shawl off your head and nearly takes you with it.
You bow into the wind and press on.
You walk along the road north past low fields to the crossroads.
(13 minutes on foot) (slowed by the weather from 13 to 26 minutes)

The wind drives the rain sideways; every hedge you pass sounds like it
is tearing itself apart.
You walk along the Kilteevan road heading south past low fields.
(13 minutes on foot) (slowed by the weather from 13 to 26 minutes)

Somewhere inland a tree goes over with a crack like a musket shot.
You walk along the road north past low fields to the crossroads.
(13 minutes on foot) (slowed by the weather from 13 to 26 minutes)
```

Storm abort (two of six attempts aborted on same seed — expected ~2.1 at 35%):

```
The wind drives the rain sideways; every hedge you pass sounds like it
is tearing itself apart.
You turn back. The storm has the better of it; you'll try again later.
(6 minutes lost to the attempt.)

Somewhere inland a tree goes over with a crack like a musket shot.
You turn back. The storm has the better of it; you'll try again later.
(6 minutes lost to the attempt.)
```

Player stays at origin. Clock advanced 6 minutes (half of nominal 13).

### Kill-switch

```
> Feature 'weather-travel' disabled.

You walk along the Kilteevan road heading south past low fields.
(13 minutes on foot)
```

Storm still in description but feature silent and 13 minutes is 13 minutes. Flag is genuine kill-switch.

## Multiplier Table Verified

| Weather | Multiplier | Observed (13 min base) |
|---|---:|---:|
| Clear / Partly Cloudy / Overcast | 1.00 | 13 min |
| Light Rain | 1.10 | 15 min |
| Heavy Rain | 1.40 | 19 min |
| Fog | 1.50 | 20 min |
| Storm | 2.00 | 26 min |

Fog > Heavy Rain ordering confirmed (visibility beats wetness).

## Check Output

```
cargo clippy -p parish-world -p parish --all-targets -- -D warnings
# clean

cargo test -p parish-world -p parish
# 440 passing, 0 failed (includes 13 new weather_travel tests)
```
