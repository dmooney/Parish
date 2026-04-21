//! Pattern-day pilgrimage at the Holy Well.
//!
//! On the feast day of a holy well's patron saint, pilgrims gather from
//! the surrounding country to make the *turas* — "the rounds" — and
//! bring news from outside the parish. This module bridges the existing
//! festival calendar and the Holy Well location by turning a quiet
//! clearing in the ash grove into a day-long gathering.
//!
//! The first (and for now only) pattern day is **Imbolc / St. Brigid's
//! Day** (February 1) at **The Holy Well** (`LocationId(17)`). The
//! feature surfaces the moment in three beats:
//!
//! 1. **Arrival** — fired once at dawn on Feb 1 when the player is at
//!    the well or the adjacent village. The clearing fills.
//! 2. **Rumours** — between arrival and dusk, spaced roughly every two
//!    game-hours, pilgrims speak "news from away" drawn from a list of
//!    1820s-Ireland one-liners (emigration, Catholic Emancipation, tithe
//!    resistance, market prices). The rumour text for a given year is
//!    deterministic — same date always picks the same lines in the same
//!    order — so play-tests and regression fixtures stay stable.
//! 3. **Departure** — fired once at dusk. The pilgrims drift off down
//!    the road; the place is quiet again.
//!
//! The whole system is gated behind the default-on `pilgrimage` feature
//! flag. When disabled, the well stays empty whatever the calendar says.

use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

use parish_types::LocationId;

/// Location id of The Holy Well in the default Rundale mod.
pub const WELL_LOCATION_ID: LocationId = LocationId(17);

/// Location id of Kilteevan Village — the village beside the well.
///
/// The village is close enough that the player overhears the gathering
/// even without walking down the mossy path, so we consider them "near"
/// for the purposes of firing scene beats.
pub const VILLAGE_LOCATION_ID: LocationId = LocationId(15);

/// Calendar month of the patron-saint feast day (February).
pub const PATTERN_MONTH: u32 = 2;
/// Calendar day of the patron-saint feast day (1st).
pub const PATTERN_DAY: u32 = 1;

/// Display name of the saint whose well this is.
pub const SAINT_NAME: &str = "St. Brigid";

/// First game-hour at which the arrival scene may fire (dawn).
pub const ARRIVAL_HOUR: u8 = 6;
/// Minimum spacing between rumours, in game-hours.
pub const RUMOUR_INTERVAL_HOURS: u8 = 2;
/// First game-hour after which a rumour may fire.
pub const RUMOUR_START_HOUR: u8 = 8;
/// Last game-hour at which a rumour may fire.
pub const RUMOUR_END_HOUR: u8 = 17;
/// Maximum rumours that can fire on a single pattern day.
pub const MAX_RUMOURS_PER_DAY: u8 = 4;
/// First game-hour at which the departure scene may fire (dusk).
pub const DEPARTURE_HOUR: u8 = 18;

/// Period-appropriate "news from away" rumours spoken by visiting pilgrims.
///
/// All lines are 1820s-Ireland in scope — emigration, Daniel O'Connell's
/// emancipation campaign, the tithe agitation, prices at Athlone and
/// Ballinasloe, fever, moonlighting, hedge schools. Kept deliberately
/// terse so the cumulative effect is "window to the wider world" rather
/// than "exposition dump".
pub const RUMOURS: &[&str] = &[
    "A pilgrim from Leitrim says it: \u{201c}A cousin's letter from Albany \
     — Irishmen are got on the canals there for a dollar a day, and meat \
     twice in the week.\u{201d}",
    "A thin man leaning on a blackthorn says: \u{201c}Mr O'Connell spoke \
     at Carrick before Christmas. A thousand gathered, and not one of them \
     wanting a drink to make him listen.\u{201d}",
    "A woman with a basket says: \u{201c}Butter's at sixpence the pound \
     at Athlone market last Thursday. We brought two firkins down and sold \
     the lot before noon.\u{201d}",
    "An old man with a rosary counts through it and says: \u{201c}My \
     brother's boy sailed out of the Cove last Saturday. Bound for Quebec. \
     His mother hasn't spoken a word since.\u{201d}",
    "A wiry young fellow lowers his voice: \u{201c}The tithe-proctor was \
     turned back from a townland beyond Strokestown. No blood spilt, only \
     sticks in the hands. But a warning.\u{201d}",
    "A stout woman with a shawl pulled high says: \u{201c}There's a fever \
     in Sligo town. Three houses on the one lane closed up, with a cross \
     chalked on the door.\u{201d}",
    "A pale girl whispers to the stone: \u{201c}A woman in Drumshanbo was \
     cured of the blindness here last pattern day. Walked home seeing her \
     own feet for the first time in ten year.\u{201d}",
    "A weathered man with clay on his boots says: \u{201c}The assizes at \
     Roscommon have two men up for moonlighting. The judge come down from \
     England and doesn't a word of Irish on him.\u{201d}",
    "A quiet old priest says, half to himself: \u{201c}New Mass cards have \
     come up from Dublin, printed on the good paper. There's a picture of \
     Brigid on the back with her cross, and she smiling.\u{201d}",
    "A red-faced drover says: \u{201c}Cattle were worth nothing at \
     Ballinasloe last fair. I drove twenty up and brought seventeen home \
     again. I'd have burnt them for the hides only the fires wouldn't \
     take.\u{201d}",
    "A thin schoolmaster in a too-big coat says: \u{201c}They scattered a \
     hedge school near Boyle last month. The master got away over a ditch \
     with a Virgil under one arm and a grammar under the other.\u{201d}",
    "A grey-haired woman crossing herself says: \u{201c}The landlord at \
     Kilmore has gone back to London for the winter and left the agent in \
     charge. God help the tenants who haven't paid by March.\u{201d}",
];

/// Which part of the pattern-day arc we are currently in, based on the
/// date and hour alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternWindow {
    /// Not a pattern day at all.
    NotPatternDay,
    /// Pattern day, but before the arrival hour.
    PreDawn,
    /// Pattern day, arrival hour — the gathering is forming.
    Arrival,
    /// Pattern day, between arrival and dusk — pilgrims present.
    Daytime,
    /// Pattern day, dusk hour — the gathering is breaking up.
    Dusk,
    /// Pattern day, after dusk — the well is quiet again.
    PostDusk,
}

impl PatternWindow {
    /// Returns `true` if the current window is "pilgrims are here" —
    /// i.e. after arrival, through daytime, up to and including dusk.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            PatternWindow::Arrival | PatternWindow::Daytime | PatternWindow::Dusk
        )
    }
}

/// Computes the pattern window for a given game time.
pub fn pattern_window(now: DateTime<Utc>) -> PatternWindow {
    let date = now.date_naive();
    if date.month() != PATTERN_MONTH || date.day() != PATTERN_DAY {
        return PatternWindow::NotPatternDay;
    }
    let hour = now.hour() as u8;
    if hour < ARRIVAL_HOUR {
        PatternWindow::PreDawn
    } else if hour == ARRIVAL_HOUR {
        PatternWindow::Arrival
    } else if hour < DEPARTURE_HOUR {
        PatternWindow::Daytime
    } else if hour == DEPARTURE_HOUR {
        PatternWindow::Dusk
    } else {
        PatternWindow::PostDusk
    }
}

/// Per-day state tracking which beats have already fired.
///
/// One of these is kept for each `NaiveDate` on which the pilgrimage
/// system has emitted anything, so beats fire at most once per day.
#[derive(Debug, Clone, Default)]
pub struct PilgrimageDayState {
    /// Whether the arrival scene has been emitted today.
    pub arrival_fired: bool,
    /// Whether the departure scene has been emitted today.
    pub departure_fired: bool,
    /// Count of rumours emitted today (capped at [`MAX_RUMOURS_PER_DAY`]).
    pub rumours_fired: u8,
    /// Game-hour (0..=23) of the most recent rumour, for cadence spacing.
    pub last_rumour_hour: Option<u8>,
}

/// A single narrative beat produced by [`tick_pilgrimage`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PilgrimageBeat {
    /// Pilgrims have arrived at the well.
    Arrival,
    /// A pilgrim speaks a one-line rumour from outside the parish.
    Rumour {
        /// Zero-based index into [`RUMOURS`].
        index: usize,
    },
    /// The gathering breaks up as dusk falls.
    Departure,
}

/// Accumulated outcome of one pilgrimage tick.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PilgrimageReport {
    /// Beats emitted this tick (may be empty).
    pub beats: Vec<PilgrimageBeat>,
}

impl PilgrimageReport {
    /// Returns `true` if nothing happened this tick.
    pub fn is_empty(&self) -> bool {
        self.beats.is_empty()
    }
}

/// Picks a rumour for the given date and slot.
///
/// Deterministic: the same `(date, slot)` always returns the same index.
/// Uses a simple mix of the year, ordinal day, and slot so that different
/// pattern days of different years surface different lines.
pub fn pick_rumour_index(date: NaiveDate, slot: u8) -> usize {
    let seed = (date.year().unsigned_abs() as usize)
        .wrapping_mul(31)
        .wrapping_add(date.ordinal() as usize)
        .wrapping_mul(7)
        .wrapping_add(slot as usize);
    seed % RUMOURS.len()
}

/// Is the player considered "near" the well for pilgrimage purposes?
///
/// True when the player is at the well itself or in the adjacent
/// Kilteevan Village (the well lane is visible from the street, and
/// the hum of a crowd two hundred yards off is unmistakable).
pub fn is_near_well(player_loc: LocationId) -> bool {
    player_loc == WELL_LOCATION_ID || player_loc == VILLAGE_LOCATION_ID
}

/// Default line for the arrival scene.
///
/// Two voicings. Near the well (player at the well itself or in the
/// adjacent village) we describe the crowd directly. Elsewhere the
/// gathering is only a murmur carried on the wind.
pub fn arrival_line(near_well: bool) -> String {
    if near_well {
        format!(
            "The pattern day has come. Pilgrims are gathering on the mossy \
             path to the well \u{2014} women with shawls over their heads, \
             men leaning on blackthorn sticks, a few barefoot children \
             pressed close. They have walked from Leitrim, from Longford, \
             from the far side of the Shannon. The old stones of {}'s well \
             are already hung with fresh rags. Someone begins the first round.",
            SAINT_NAME
        )
    } else {
        format!(
            "Somewhere beyond the village, on the mossy path to {}'s well, \
             people are gathering. You can hear them only faintly \u{2014} the \
             murmur of many voices carried on the morning wind.",
            SAINT_NAME
        )
    }
}

/// Default line for the departure scene.
pub fn departure_line(near_well: bool) -> String {
    if near_well {
        format!(
            "The light is going out of the ash grove. One by one the \
             pilgrims finish their rounds, kiss the stone, and start back \
             down the road in twos and threes. Someone calls a blessing \
             back over their shoulder. Within the hour, {}'s well is quiet \
             again \u{2014} rags and ribbons moving in the wind, but no \
             voices.",
            SAINT_NAME
        )
    } else {
        "The sound from beyond the village fades as dusk takes the sky. \
         The pilgrims are starting for home."
            .to_string()
    }
}

/// Formats a rumour line for the given rumour index.
///
/// The returned string is the raw rumour prefixed with a short scene
/// cue so the player knows a pilgrim is speaking.
pub fn rumour_line(index: usize, near_well: bool) -> String {
    let raw = RUMOURS
        .get(index)
        .copied()
        .unwrap_or("A pilgrim crosses themselves and says a prayer under their breath.");
    if near_well {
        raw.to_string()
    } else {
        format!("From the direction of the well, a voice carries: {}", raw)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(month: u32, day: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(1820, month, day, h, 0, 0).unwrap()
    }

    #[test]
    fn not_pattern_day_on_ordinary_date() {
        assert_eq!(pattern_window(t(3, 20, 8)), PatternWindow::NotPatternDay);
    }

    #[test]
    fn pre_dawn_before_six() {
        assert_eq!(pattern_window(t(2, 1, 5)), PatternWindow::PreDawn);
    }

    #[test]
    fn arrival_at_six() {
        assert_eq!(pattern_window(t(2, 1, 6)), PatternWindow::Arrival);
    }

    #[test]
    fn daytime_between_seven_and_seventeen() {
        assert_eq!(pattern_window(t(2, 1, 7)), PatternWindow::Daytime);
        assert_eq!(pattern_window(t(2, 1, 12)), PatternWindow::Daytime);
        assert_eq!(pattern_window(t(2, 1, 17)), PatternWindow::Daytime);
    }

    #[test]
    fn dusk_at_eighteen() {
        assert_eq!(pattern_window(t(2, 1, 18)), PatternWindow::Dusk);
    }

    #[test]
    fn post_dusk_after_eighteen() {
        assert_eq!(pattern_window(t(2, 1, 19)), PatternWindow::PostDusk);
        assert_eq!(pattern_window(t(2, 1, 23)), PatternWindow::PostDusk);
    }

    #[test]
    fn is_active_covers_arrival_through_dusk() {
        assert!(!PatternWindow::NotPatternDay.is_active());
        assert!(!PatternWindow::PreDawn.is_active());
        assert!(PatternWindow::Arrival.is_active());
        assert!(PatternWindow::Daytime.is_active());
        assert!(PatternWindow::Dusk.is_active());
        assert!(!PatternWindow::PostDusk.is_active());
    }

    #[test]
    fn pick_rumour_is_deterministic_for_same_inputs() {
        let d = NaiveDate::from_ymd_opt(1820, 2, 1).unwrap();
        assert_eq!(pick_rumour_index(d, 0), pick_rumour_index(d, 0));
        assert_eq!(pick_rumour_index(d, 1), pick_rumour_index(d, 1));
    }

    #[test]
    fn pick_rumour_stays_in_bounds() {
        for slot in 0..16u8 {
            let d = NaiveDate::from_ymd_opt(1820, 2, 1).unwrap();
            assert!(pick_rumour_index(d, slot) < RUMOURS.len());
        }
    }

    #[test]
    fn pick_rumour_differs_across_slots_same_day() {
        let d = NaiveDate::from_ymd_opt(1820, 2, 1).unwrap();
        // For a 12-entry list, the first four slots should be unique in
        // the seed scheme above.
        let indices: Vec<usize> = (0..4).map(|s| pick_rumour_index(d, s)).collect();
        let unique: std::collections::HashSet<_> = indices.iter().copied().collect();
        assert_eq!(
            unique.len(),
            4,
            "expected four distinct rumours, got {indices:?}"
        );
    }

    #[test]
    fn near_well_covers_both_well_and_village() {
        assert!(is_near_well(WELL_LOCATION_ID));
        assert!(is_near_well(VILLAGE_LOCATION_ID));
        assert!(!is_near_well(LocationId(1))); // crossroads
    }

    #[test]
    fn arrival_line_mentions_saint() {
        assert!(arrival_line(true).contains("St. Brigid"));
        assert!(arrival_line(false).contains("St. Brigid"));
    }

    #[test]
    fn departure_line_differs_by_proximity() {
        let near = departure_line(true);
        let far = departure_line(false);
        assert_ne!(near, far);
        assert!(near.contains("St. Brigid"));
    }

    #[test]
    fn rumour_line_wraps_when_far() {
        let line = rumour_line(0, false);
        assert!(line.starts_with("From the direction of the well"));
    }

    #[test]
    fn report_is_empty_on_default() {
        assert!(PilgrimageReport::default().is_empty());
    }
}
