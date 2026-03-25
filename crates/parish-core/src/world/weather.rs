//! Weather state machine with season-dependent transition probabilities.
//!
//! The [`WeatherEngine`] drives weather changes over time using a Markov-style
//! transition matrix that varies by [`Season`]. Each tick checks whether the
//! minimum duration has elapsed and, if so, probabilistically selects the next
//! weather state.

use chrono::{DateTime, Utc};
use rand::Rng;

use crate::world::Weather;
use crate::world::time::Season;

/// Number of [`Weather`] variants.
const WEATHER_COUNT: usize = 7;

/// All weather variants in enum order, for indexed access.
const ALL_WEATHERS: [Weather; WEATHER_COUNT] = [
    Weather::Clear,
    Weather::PartlyCloudy,
    Weather::Overcast,
    Weather::LightRain,
    Weather::Rain,
    Weather::Fog,
    Weather::Storm,
];

/// Drives weather transitions over game time.
///
/// Holds the current weather, the game-time instant it was set, and a
/// minimum hold duration. On each [`tick`](WeatherEngine::tick), the engine
/// checks elapsed game-hours and probabilistically transitions to a new
/// state based on the current [`Season`].
pub struct WeatherEngine {
    /// The current weather condition.
    current: Weather,
    /// Game-time instant when the current weather began.
    since: DateTime<Utc>,
    /// Minimum game-hours before a transition is considered.
    min_duration_hours: u32,
}

impl WeatherEngine {
    /// Creates a new engine starting with `initial` weather at game-time `now`.
    ///
    /// The default minimum hold duration is 2 game-hours.
    pub fn new(initial: Weather, now: DateTime<Utc>) -> Self {
        Self {
            current: initial,
            since: now,
            min_duration_hours: 2,
        }
    }

    /// Returns the current weather.
    pub fn current(&self) -> Weather {
        self.current
    }

    /// Returns the game-time instant when the current weather began.
    pub fn since(&self) -> DateTime<Utc> {
        self.since
    }

    /// Attempts a weather transition.
    ///
    /// Returns `Some(new_weather)` if the weather changed, or `None` if the
    /// minimum duration has not elapsed or the roll selected the same state.
    pub fn tick(
        &mut self,
        now: DateTime<Utc>,
        season: Season,
        rng: &mut impl Rng,
    ) -> Option<Weather> {
        let elapsed = now.signed_duration_since(self.since);
        let min = chrono::Duration::hours(i64::from(self.min_duration_hours));
        if elapsed < min {
            return None;
        }

        let weights = transition_weights(self.current, season);
        let new = weighted_pick(&weights, rng);

        if new != self.current {
            self.current = new;
            self.since = now;
            Some(new)
        } else {
            None
        }
    }
}

/// Picks a [`Weather`] variant from weighted pairs using a uniform roll.
fn weighted_pick(weights: &[(Weather, f32); WEATHER_COUNT], rng: &mut impl Rng) -> Weather {
    let total: f32 = weights.iter().map(|(_, w)| w).sum();
    let mut roll = rng.r#gen::<f32>() * total;

    for &(weather, w) in weights {
        roll -= w;
        if roll <= 0.0 {
            return weather;
        }
    }

    // Fallback (should not be reached with valid weights).
    weights[WEATHER_COUNT - 1].0
}

/// Returns season-dependent transition weights from a given weather state.
///
/// Each entry is `(target_weather, weight)`. Weights need not sum to 1.0;
/// they are normalised at selection time.
fn transition_weights(from: Weather, season: Season) -> [(Weather, f32); WEATHER_COUNT] {
    match season {
        Season::Summer => summer_weights(from),
        Season::Winter => winter_weights(from),
        Season::Autumn => autumn_weights(from),
        Season::Spring => spring_weights(from),
    }
}

// ---------------------------------------------------------------------------
// Season-specific weight tables
// ---------------------------------------------------------------------------

/// Summer: predominantly clear, storms are brief.
fn summer_weights(from: Weather) -> [(Weather, f32); WEATHER_COUNT] {
    use Weather::*;
    match from {
        Clear => weighted_row([0.50, 0.25, 0.10, 0.05, 0.02, 0.05, 0.03]),
        PartlyCloudy => weighted_row([0.30, 0.30, 0.20, 0.10, 0.03, 0.04, 0.03]),
        Overcast => weighted_row([0.10, 0.20, 0.30, 0.20, 0.05, 0.10, 0.05]),
        LightRain => weighted_row([0.10, 0.15, 0.25, 0.25, 0.10, 0.10, 0.05]),
        Rain => weighted_row([0.05, 0.10, 0.20, 0.30, 0.20, 0.05, 0.10]),
        Fog => weighted_row([0.20, 0.20, 0.20, 0.10, 0.05, 0.20, 0.05]),
        Storm => weighted_row([0.05, 0.10, 0.30, 0.20, 0.20, 0.05, 0.10]),
    }
}

/// Winter: more overcast, rain, and storms; less clear.
fn winter_weights(from: Weather) -> [(Weather, f32); WEATHER_COUNT] {
    use Weather::*;
    match from {
        Clear => weighted_row([0.20, 0.30, 0.20, 0.10, 0.08, 0.07, 0.05]),
        PartlyCloudy => weighted_row([0.10, 0.20, 0.30, 0.15, 0.10, 0.08, 0.07]),
        Overcast => weighted_row([0.05, 0.10, 0.30, 0.20, 0.15, 0.10, 0.10]),
        LightRain => weighted_row([0.03, 0.07, 0.20, 0.25, 0.20, 0.10, 0.15]),
        Rain => weighted_row([0.02, 0.05, 0.15, 0.20, 0.30, 0.08, 0.20]),
        Fog => weighted_row([0.05, 0.10, 0.25, 0.15, 0.10, 0.25, 0.10]),
        Storm => weighted_row([0.02, 0.05, 0.20, 0.15, 0.25, 0.08, 0.25]),
    }
}

/// Autumn: fog and rain are common; moderate storms.
fn autumn_weights(from: Weather) -> [(Weather, f32); WEATHER_COUNT] {
    use Weather::*;
    match from {
        Clear => weighted_row([0.25, 0.30, 0.18, 0.10, 0.05, 0.08, 0.04]),
        PartlyCloudy => weighted_row([0.15, 0.20, 0.25, 0.15, 0.08, 0.10, 0.07]),
        Overcast => weighted_row([0.05, 0.12, 0.28, 0.22, 0.12, 0.12, 0.09]),
        LightRain => weighted_row([0.05, 0.08, 0.20, 0.25, 0.18, 0.12, 0.12]),
        Rain => weighted_row([0.03, 0.05, 0.15, 0.22, 0.28, 0.10, 0.17]),
        Fog => weighted_row([0.08, 0.12, 0.20, 0.15, 0.10, 0.28, 0.07]),
        Storm => weighted_row([0.03, 0.07, 0.22, 0.18, 0.22, 0.08, 0.20]),
    }
}

/// Spring: moderate, similar to summer but with slightly more rain.
fn spring_weights(from: Weather) -> [(Weather, f32); WEATHER_COUNT] {
    use Weather::*;
    match from {
        Clear => weighted_row([0.40, 0.25, 0.15, 0.08, 0.04, 0.05, 0.03]),
        PartlyCloudy => weighted_row([0.20, 0.28, 0.22, 0.13, 0.05, 0.07, 0.05]),
        Overcast => weighted_row([0.08, 0.15, 0.30, 0.22, 0.08, 0.10, 0.07]),
        LightRain => weighted_row([0.08, 0.12, 0.22, 0.25, 0.15, 0.10, 0.08]),
        Rain => weighted_row([0.04, 0.08, 0.18, 0.25, 0.25, 0.07, 0.13]),
        Fog => weighted_row([0.15, 0.15, 0.22, 0.12, 0.07, 0.22, 0.07]),
        Storm => weighted_row([0.05, 0.08, 0.25, 0.20, 0.22, 0.05, 0.15]),
    }
}

/// Helper to pair weight values with their corresponding [`Weather`] variant.
fn weighted_row(weights: [f32; WEATHER_COUNT]) -> [(Weather, f32); WEATHER_COUNT] {
    let mut result = [(Weather::Clear, 0.0f32); WEATHER_COUNT];
    for (i, &w) in weights.iter().enumerate() {
        result[i] = (ALL_WEATHERS[i], w);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    fn make_time(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(1820, 6, 15, hour, 0, 0).unwrap()
    }

    #[test]
    fn test_weather_engine_new() {
        let now = make_time(8);
        let engine = WeatherEngine::new(Weather::Clear, now);
        assert_eq!(engine.current(), Weather::Clear);
        assert_eq!(engine.since(), now);
    }

    #[test]
    fn test_min_duration_respected() {
        let now = make_time(8);
        let mut engine = WeatherEngine::new(Weather::Clear, now);
        let mut rng = StdRng::seed_from_u64(42);

        // 1 hour later — should be blocked
        let later = now + chrono::Duration::hours(1);
        assert!(engine.tick(later, Season::Summer, &mut rng).is_none());
    }

    #[test]
    fn test_weather_can_change() {
        let now = make_time(8);
        let mut engine = WeatherEngine::new(Weather::Clear, now);
        let mut rng = StdRng::seed_from_u64(42);

        // Tick many times 3 hours apart until we see a change
        let mut changed = false;
        let mut t = now;
        for _ in 0..100 {
            t = t + chrono::Duration::hours(3);
            if engine.tick(t, Season::Winter, &mut rng).is_some() {
                changed = true;
                break;
            }
        }
        assert!(changed, "Weather should eventually change");
    }

    #[test]
    fn test_all_transitions_valid() {
        let mut rng = StdRng::seed_from_u64(123);
        let seasons = [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ];
        for &from in &ALL_WEATHERS {
            for &season in &seasons {
                let weights = transition_weights(from, season);
                for _ in 0..50 {
                    let picked = weighted_pick(&weights, &mut rng);
                    assert!(
                        ALL_WEATHERS.contains(&picked),
                        "picked invalid weather: {picked:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_seasonal_bias() {
        // Run many transitions from Clear and count rain-family states.
        // Winter should produce more rain than summer.
        let mut rng_winter = StdRng::seed_from_u64(999);
        let mut rng_summer = StdRng::seed_from_u64(999);
        let iterations = 10_000;

        let winter_weights = transition_weights(Weather::Clear, Season::Winter);
        let summer_weights = transition_weights(Weather::Clear, Season::Summer);

        let mut winter_rain = 0u32;
        let mut summer_rain = 0u32;

        for _ in 0..iterations {
            let w = weighted_pick(&winter_weights, &mut rng_winter);
            if matches!(w, Weather::LightRain | Weather::Rain | Weather::Storm) {
                winter_rain += 1;
            }
            let s = weighted_pick(&summer_weights, &mut rng_summer);
            if matches!(s, Weather::LightRain | Weather::Rain | Weather::Storm) {
                summer_rain += 1;
            }
        }

        assert!(
            winter_rain > summer_rain,
            "Winter should have more rain-family states than summer: \
             winter={winter_rain}, summer={summer_rain}"
        );
    }

    #[test]
    fn test_weights_sum_approximately_one() {
        let seasons = [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ];
        for &from in &ALL_WEATHERS {
            for &season in &seasons {
                let weights = transition_weights(from, season);
                let total: f32 = weights.iter().map(|(_, w)| w).sum();
                assert!(
                    (total - 1.0).abs() < 0.02,
                    "Weights for {from:?}/{season:?} should sum to ~1.0, got {total}"
                );
            }
        }
    }

    #[test]
    fn test_weighted_pick_deterministic() {
        let weights = weighted_row([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let mut rng = StdRng::seed_from_u64(0);
        for _ in 0..100 {
            assert_eq!(weighted_pick(&weights, &mut rng), Weather::Clear);
        }
    }

    #[test]
    fn test_engine_updates_since_on_change() {
        let now = make_time(8);
        let mut engine = WeatherEngine::new(Weather::Storm, now);
        let mut rng = StdRng::seed_from_u64(42);

        // Keep ticking until weather changes
        let mut t = now;
        for _ in 0..200 {
            t = t + chrono::Duration::hours(3);
            if let Some(_new) = engine.tick(t, Season::Summer, &mut rng) {
                assert_eq!(engine.since(), t);
                return;
            }
        }
        panic!("Weather never changed in 200 ticks");
    }
}
