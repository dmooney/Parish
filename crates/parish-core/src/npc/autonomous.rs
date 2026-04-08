//! Autonomous NPC speech selection.
//!
//! After a player turn (and any addressed NPCs respond), the dispatcher may
//! run a short autonomous chain where NPCs at the player's location react
//! to each other for a couple of turns. The same machinery is reused for
//! "spontaneous speech" — if the player has been idle for a while, an NPC
//! at their location may speak unprompted.
//!
//! [`pick_next_speaker`] is the heuristic that decides who, if anyone,
//! should speak next. It is intentionally pure (no I/O, no async) so the
//! per-mode dispatch loops can call it inside a single lock scope.

use crate::npc::{Npc, NpcId};
use crate::world::LocationId;

/// Maximum number of autonomous turns in a single NPC-to-NPC chain.
///
/// After an addressed-NPC turn completes, the dispatcher may run up to this
/// many additional NPC-to-NPC exchanges before yielding the floor back to
/// the player. Keeps cost bounded and prevents runaway loops.
pub const MAX_CHAIN_TURNS: usize = 3;

/// Score threshold below which no NPC speaks up. Computed as the sum of
/// the heuristics in [`pick_next_speaker`]; an NPC with only the baseline
/// score scores 0.4 and stays silent.
pub const SPEAK_UP_THRESHOLD: f32 = 0.5;

/// Picks the next NPC who should speak in an autonomous chain, or `None`
/// if no one feels strongly enough to chime in.
///
/// Heuristic (v1, intentionally simple — refine in a follow-up):
/// 1. Eligible candidates: NPCs in `npcs_at_location` whose id is NOT in
///    `recently_spoken` and is NOT the `last_speaker_id` (no one talks to
///    themselves twice in a row).
/// 2. For each candidate, compute a "wants to speak" score:
///    - +0.4 baseline (everyone has a small chance).
///    - +0.3 if they have a non-neutral relationship with the last speaker.
///    - +0.2 if they were just addressed by the player (they get to react).
///    - +0.1 if their mood is high-energy.
/// 3. Pick the highest-scoring NPC. If their score is below
///    [`SPEAK_UP_THRESHOLD`], return `None`.
///
/// Ties are broken by NPC id (lowest first) for determinism. Pass a
/// shuffled `npcs_at_location` slice to the caller if you want randomness.
pub fn pick_next_speaker<'a>(
    npcs_at_location: &[&'a Npc],
    last_speaker_id: Option<NpcId>,
    recently_spoken: &[NpcId],
    addressed_this_turn: &[NpcId],
) -> Option<&'a Npc> {
    let mut best: Option<(&'a Npc, f32)> = None;

    for npc in npcs_at_location {
        // Exclude the most recent speaker — they don't immediately talk
        // back to themselves.
        if Some(npc.id) == last_speaker_id {
            continue;
        }
        // Exclude NPCs who have already taken a turn in this chain.
        if recently_spoken.contains(&npc.id) {
            continue;
        }

        let mut score: f32 = 0.4; // baseline

        // Bonus: has a non-neutral relationship with the last speaker.
        if let Some(prev_id) = last_speaker_id
            && let Some(rel) = npc.relationships.get(&prev_id)
            && rel.strength.abs() > 0.1
        {
            score += 0.3;
        }

        // Bonus: was directly addressed by the player this turn.
        if addressed_this_turn.contains(&npc.id) {
            score += 0.2;
        }

        // Bonus: high-energy mood (more likely to chime in).
        if is_high_energy_mood(&npc.mood) {
            score += 0.1;
        }

        match best {
            None => best = Some((npc, score)),
            Some((_, prev_score)) if score > prev_score => best = Some((npc, score)),
            _ => {}
        }
    }

    best.filter(|(_, score)| *score >= SPEAK_UP_THRESHOLD)
        .map(|(npc, _)| npc)
}

/// Returns true for moods that suggest an NPC is more likely to speak up.
fn is_high_energy_mood(mood: &str) -> bool {
    let lower = mood.to_lowercase();
    matches!(
        lower.as_str(),
        "excited"
            | "agitated"
            | "joyful"
            | "angry"
            | "indignant"
            | "outraged"
            | "elated"
            | "boisterous"
            | "anxious"
            | "scared"
    )
}

/// Determines whether an autonomous chain should continue based on the
/// number of turns already taken and whether any candidate exists.
pub fn should_continue_chain(turns_taken: usize, candidate: Option<&Npc>) -> bool {
    turns_taken < MAX_CHAIN_TURNS && candidate.is_some()
}

/// Returns the player's current location id from a [`crate::world::WorldState`].
/// (Tiny helper kept here for symmetry with the rest of the chain logic.)
#[inline]
pub fn player_location(world: &crate::world::WorldState) -> LocationId {
    world.player_location
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npc::types::{Relationship, RelationshipKind};
    use std::collections::HashMap;

    fn make_npc(id: u32, name: &str, mood: &str) -> Npc {
        let mut npc = Npc::new_test_npc();
        npc.id = NpcId(id);
        npc.name = name.to_string();
        npc.mood = mood.to_string();
        npc.relationships = HashMap::new();
        npc
    }

    #[test]
    fn pick_returns_none_when_only_last_speaker_present() {
        let a = make_npc(1, "A", "calm");
        let candidates: Vec<&Npc> = vec![&a];
        let pick = pick_next_speaker(&candidates, Some(NpcId(1)), &[], &[]);
        assert!(pick.is_none(), "the only NPC is the last speaker → no pick");
    }

    #[test]
    fn pick_excludes_recently_spoken() {
        let a = make_npc(1, "A", "calm");
        let b = make_npc(2, "B", "calm");
        let candidates: Vec<&Npc> = vec![&a, &b];
        // Both have only the baseline 0.4 score → below threshold
        let pick = pick_next_speaker(&candidates, None, &[NpcId(1)], &[]);
        assert!(pick.is_none(), "B's baseline 0.4 is under the threshold");
    }

    #[test]
    fn pick_returns_addressed_candidate_over_silent_baseline() {
        let mut a = make_npc(1, "A", "calm");
        let b = make_npc(2, "B", "calm");
        // Give A a relationship to no one — only baseline.
        // Make B "addressed" so they score 0.4 + 0.2 = 0.6.
        let candidates: Vec<&Npc> = vec![&a, &b];
        let pick = pick_next_speaker(&candidates, None, &[], &[NpcId(2)]);
        assert!(pick.is_some());
        assert_eq!(pick.unwrap().id, NpcId(2));
        // Touch A so it isn't unused.
        a.mood = "calm".to_string();
    }

    #[test]
    fn pick_prefers_npc_with_relationship_to_last_speaker() {
        let mut a = make_npc(1, "A", "calm");
        let b = make_npc(2, "B", "calm");
        let last_speaker = NpcId(99);
        // A has a strong relationship to the last speaker → +0.3 → 0.7
        // B has no relationship → 0.4 (below threshold)
        a.relationships.insert(
            last_speaker,
            Relationship::new(RelationshipKind::Friend, 0.8),
        );
        let candidates: Vec<&Npc> = vec![&a, &b];
        let pick = pick_next_speaker(&candidates, Some(last_speaker), &[], &[]);
        assert!(pick.is_some());
        assert_eq!(pick.unwrap().id, NpcId(1));
    }

    #[test]
    fn pick_high_energy_mood_pushes_over_threshold_when_addressed() {
        let a = make_npc(1, "A", "excited");
        let candidates: Vec<&Npc> = vec![&a];
        // baseline 0.4 + addressed 0.2 + high-energy 0.1 = 0.7 ≥ threshold
        let pick = pick_next_speaker(&candidates, None, &[], &[NpcId(1)]);
        assert!(pick.is_some());
    }

    #[test]
    fn pick_calm_mood_alone_stays_silent() {
        let a = make_npc(1, "A", "calm");
        let candidates: Vec<&Npc> = vec![&a];
        // Only baseline 0.4 → below threshold
        let pick = pick_next_speaker(&candidates, None, &[], &[]);
        assert!(pick.is_none());
    }

    #[test]
    fn should_continue_chain_caps_at_max_turns() {
        let a = make_npc(1, "A", "calm");
        assert!(should_continue_chain(0, Some(&a)));
        assert!(should_continue_chain(MAX_CHAIN_TURNS - 1, Some(&a)));
        assert!(!should_continue_chain(MAX_CHAIN_TURNS, Some(&a)));
        // Even with turns left, no candidate → stop
        assert!(!should_continue_chain(0, None));
    }

    #[test]
    fn is_high_energy_mood_recognises_common_moods() {
        assert!(is_high_energy_mood("excited"));
        assert!(is_high_energy_mood("EXCITED"));
        assert!(is_high_energy_mood("agitated"));
        assert!(is_high_energy_mood("joyful"));
        assert!(!is_high_energy_mood("calm"));
        assert!(!is_high_energy_mood("content"));
        assert!(!is_high_energy_mood(""));
    }
}
