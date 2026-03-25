//! Weather state machine with season-dependent Markov chain transitions.
//!
//! The weather engine uses a Markov chain with season-dependent transition
//! weights to simulate realistic weather progression. A minimum duration
//! prevents unrealistically rapid weather changes.

use super::Weather;
use super::time::Season;
use chrono::{DateTime, Utc};
use rand::Rng;

/// All weather states for iteration.
#[cfg(test)]
const ALL_WEATHERS: [Weather; 7] = [
    Weather::Clear,
    Weather::PartlyCloudy,
    Weather::Overcast,
    Weather::LightRain,
    Weather::Rain,
    Weather::Fog,
    Weather::Storm,
];

/// Minimum hours between weather changes.
const MIN_DURATION_HOURS: u32 = 2;

/// Weather engine with Markov chain transitions.
///
/// Tracks the current weather state, the time it last changed, and
/// enforces a minimum duration between transitions.
pub struct WeatherEngine {
    /// Current weather condition.
    current: Weather,
    /// When the current weather began.
    since: DateTime<Utc>,
    /// Minimum hours before a transition can occur.
    min_duration_hours: u32,
}

impl WeatherEngine {
    /// Creates a new weather engine starting with the given weather at the given time.
    pub fn new(initial: Weather, now: DateTime<Utc>) -> Self {
        Self {
            current: initial,
            since: now,
            min_duration_hours: MIN_DURATION_HOURS,
        }
    }

    /// Returns the current weather.
    pub fn current(&self) -> Weather {
        self.current
    }

    /// Returns when the current weather began.
    pub fn since(&self) -> DateTime<Utc> {
        self.since
    }

    /// Advances the weather engine by one tick.
    ///
    /// If enough time has elapsed since the last change, a Markov chain
    /// transition is attempted based on the current season. Returns
    /// `Some(new_weather)` if the weather changed, `None` otherwise.
    pub fn tick(&mut self, now: DateTime<Utc>, season: Season) -> Option<Weather> {
        let elapsed = now.signed_duration_since(self.since);
        let elapsed_hours = elapsed.num_hours();

        if elapsed_hours < self.min_duration_hours as i64 {
            return None;
        }

        let weights = transition_weights(self.current, season);
        let mut rng = rand::thread_rng();
        let new_weather = weighted_pick(&weights, &mut rng);

        if new_weather != self.current {
            self.current = new_weather;
            self.since = now;
            Some(new_weather)
        } else {
            None
        }
    }
}

impl Default for WeatherEngine {
    fn default() -> Self {
        Self::new(Weather::Clear, Utc::now())
    }
}

/// Returns transition weights from the given weather state, adjusted for season.
///
/// Each entry is `(Weather, weight)`. Higher weights mean more likely transitions.
/// Autumn and winter have heavier rain/storm weights; spring and summer favor
/// clear skies.
fn transition_weights(from: Weather, season: Season) -> Vec<(Weather, f32)> {
    let base = match from {
        Weather::Clear => vec![
            (Weather::Clear, 0.50),
            (Weather::PartlyCloudy, 0.25),
            (Weather::Overcast, 0.10),
            (Weather::LightRain, 0.05),
            (Weather::Rain, 0.02),
            (Weather::Fog, 0.06),
            (Weather::Storm, 0.02),
        ],
        Weather::PartlyCloudy => vec![
            (Weather::Clear, 0.25),
            (Weather::PartlyCloudy, 0.30),
            (Weather::Overcast, 0.20),
            (Weather::LightRain, 0.10),
            (Weather::Rain, 0.05),
            (Weather::Fog, 0.05),
            (Weather::Storm, 0.05),
        ],
        Weather::Overcast => vec![
            (Weather::Clear, 0.10),
            (Weather::PartlyCloudy, 0.15),
            (Weather::Overcast, 0.30),
            (Weather::LightRain, 0.15),
            (Weather::Rain, 0.15),
            (Weather::Fog, 0.10),
            (Weather::Storm, 0.05),
        ],
        Weather::LightRain => vec![
            (Weather::Clear, 0.05),
            (Weather::PartlyCloudy, 0.10),
            (Weather::Overcast, 0.20),
            (Weather::LightRain, 0.30),
            (Weather::Rain, 0.20),
            (Weather::Fog, 0.05),
            (Weather::Storm, 0.10),
        ],
        Weather::Rain => vec![
            (Weather::Clear, 0.05),
            (Weather::PartlyCloudy, 0.05),
            (Weather::Overcast, 0.20),
            (Weather::LightRain, 0.20),
            (Weather::Rain, 0.30),
            (Weather::Fog, 0.05),
            (Weather::Storm, 0.15),
        ],
        Weather::Fog => vec![
            (Weather::Clear, 0.15),
            (Weather::PartlyCloudy, 0.15),
            (Weather::Overcast, 0.25),
            (Weather::LightRain, 0.10),
            (Weather::Rain, 0.05),
            (Weather::Fog, 0.25),
            (Weather::Storm, 0.05),
        ],
        Weather::Storm => vec![
            (Weather::Clear, 0.05),
            (Weather::PartlyCloudy, 0.05),
            (Weather::Overcast, 0.20),
            (Weather::LightRain, 0.10),
            (Weather::Rain, 0.30),
            (Weather::Fog, 0.05),
            (Weather::Storm, 0.25),
        ],
    };

    // Apply seasonal modifiers
    let rain_mult = match season {
        Season::Spring => 1.1,
        Season::Summer => 0.8,
        Season::Autumn => 1.4,
        Season::Winter => 1.3,
    };

    let clear_mult = match season {
        Season::Spring => 1.0,
        Season::Summer => 1.3,
        Season::Autumn => 0.7,
        Season::Winter => 0.8,
    };

    let adjusted: Vec<(Weather, f32)> = base
        .into_iter()
        .map(|(w, weight)| {
            let mult = match w {
                Weather::Clear | Weather::PartlyCloudy => clear_mult,
                Weather::Rain | Weather::LightRain | Weather::Storm => rain_mult,
                _ => 1.0,
            };
            (w, weight * mult)
        })
        .collect();

    // Normalize weights so they sum to 1.0
    let total: f32 = adjusted.iter().map(|(_, w)| w).sum();
    adjusted
        .into_iter()
        .map(|(w, wt)| (w, wt / total))
        .collect()
}

/// Picks a weather state from weighted options using the given RNG.
fn weighted_pick<R: Rng>(weights: &[(Weather, f32)], rng: &mut R) -> Weather {
    let roll: f32 = rng.r#gen::<f32>();
    let mut cumulative = 0.0;
    for &(weather, weight) in weights {
        cumulative += weight;
        if roll < cumulative {
            return weather;
        }
    }
    // Fallback to last entry (rounding edge case)
    weights.last().map(|&(w, _)| w).unwrap_or(Weather::Clear)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_weather_engine_new() {
        let now = Utc::now();
        let engine = WeatherEngine::new(Weather::Clear, now);
        assert_eq!(engine.current(), Weather::Clear);
        assert_eq!(engine.since(), now);
    }

    #[test]
    fn test_weather_engine_default() {
        let engine = WeatherEngine::default();
        assert_eq!(engine.current(), Weather::Clear);
    }

    #[test]
    fn test_tick_before_min_duration() {
        let now = Utc::now();
        let mut engine = WeatherEngine::new(Weather::Clear, now);
        // Only 1 hour later — should not change
        let later = now + Duration::hours(1);
        let result = engine.tick(later, Season::Spring);
        assert!(result.is_none());
        assert_eq!(engine.current(), Weather::Clear);
    }

    #[test]
    fn test_tick_after_min_duration() {
        let now = Utc::now();
        let later = now + Duration::hours(3);
        // Run many ticks — at least one should eventually produce a change
        // (or stay the same, both are valid outcomes of the Markov chain)
        let mut any_change = false;
        for _ in 0..100 {
            let mut e = WeatherEngine::new(Weather::Clear, now);
            if e.tick(later, Season::Autumn).is_some() {
                any_change = true;
                break;
            }
        }
        // With autumn's rain-heavy weights, we should see at least one change in 100 tries
        assert!(
            any_change,
            "Expected at least one weather change in 100 ticks"
        );
    }

    #[test]
    fn test_tick_updates_since() {
        let now = Utc::now();
        let later = now + Duration::hours(3);
        // Storm has high transition probability away from itself
        for _ in 0..100 {
            let mut engine = WeatherEngine::new(Weather::Storm, now);
            if let Some(_new) = engine.tick(later, Season::Summer) {
                assert_eq!(engine.since(), later);
                return;
            }
        }
        // If we got here, storm stayed storm 100 times — still valid
    }

    #[test]
    fn test_transition_weights_sum_to_one() {
        for &weather in &ALL_WEATHERS {
            for season in &[
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let weights = transition_weights(weather, *season);
                let sum: f32 = weights.iter().map(|(_, w)| w).sum();
                assert!(
                    (sum - 1.0).abs() < 0.01,
                    "Weights for {:?}/{:?} sum to {sum}, expected ~1.0",
                    weather,
                    season
                );
            }
        }
    }

    #[test]
    fn test_transition_weights_all_positive() {
        for &weather in &ALL_WEATHERS {
            for season in &[
                Season::Spring,
                Season::Summer,
                Season::Autumn,
                Season::Winter,
            ] {
                let weights = transition_weights(weather, *season);
                for (w, weight) in &weights {
                    assert!(
                        *weight > 0.0,
                        "Weight for {:?} -> {:?} in {:?} should be positive",
                        weather,
                        w,
                        season
                    );
                }
            }
        }
    }

    #[test]
    fn test_weighted_pick_deterministic() {
        // With weight 1.0 on a single option, should always pick it
        let weights = vec![(Weather::Fog, 1.0)];
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            assert_eq!(weighted_pick(&weights, &mut rng), Weather::Fog);
        }
    }

    #[test]
    fn test_autumn_more_rain_than_summer() {
        let autumn_weights = transition_weights(Weather::Clear, Season::Autumn);
        let summer_weights = transition_weights(Weather::Clear, Season::Summer);

        let autumn_rain: f32 = autumn_weights
            .iter()
            .filter(|(w, _)| matches!(w, Weather::Rain | Weather::LightRain | Weather::Storm))
            .map(|(_, w)| w)
            .sum();
        let summer_rain: f32 = summer_weights
            .iter()
            .filter(|(w, _)| matches!(w, Weather::Rain | Weather::LightRain | Weather::Storm))
            .map(|(_, w)| w)
            .sum();

        assert!(
            autumn_rain > summer_rain,
            "Autumn should have more rain probability ({autumn_rain}) than summer ({summer_rain})"
        );
    }

    #[test]
    fn test_all_weathers_array() {
        assert_eq!(ALL_WEATHERS.len(), 7);
        assert_eq!(ALL_WEATHERS[0], Weather::Clear);
        assert_eq!(ALL_WEATHERS[6], Weather::Storm);
    }
}
