//! Event bus for broadcasting world events to subscribers.
//!
//! Uses a `tokio::sync::broadcast` channel so multiple consumers
//! (NPC cognition tiers, UI, logging) can independently receive
//! every event without blocking the publisher.

use tokio::sync::broadcast;

use crate::npc::NpcId;
use crate::world::time::{Festival, Season};
use crate::world::{LocationId, Weather};

/// Default capacity of the broadcast channel backing the event bus.
const EVENT_BUS_CAPACITY: usize = 256;

/// A discrete event that occurs in the game world.
///
/// Published on the [`EventBus`] so that any number of subscribers
/// (NPC tick systems, UI layers, analytics) can react independently.
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// The weather has changed globally.
    WeatherChanged {
        /// Previous weather condition.
        old: Weather,
        /// New weather condition.
        new: Weather,
    },
    /// An NPC's mood label changed (e.g. from Tier-2 inference).
    NpcMoodChanged {
        /// The NPC whose mood changed.
        npc_id: NpcId,
        /// Previous mood label.
        old_mood: String,
        /// New mood label.
        new_mood: String,
    },
    /// An NPC moved between locations.
    NpcMoved {
        /// The NPC that moved.
        npc_id: NpcId,
        /// Location the NPC departed from.
        from: LocationId,
        /// Location the NPC arrived at.
        to: LocationId,
    },
    /// A piece of gossip propagated from one NPC to another.
    GossipSpread {
        /// NPC that shared the gossip.
        from_npc: NpcId,
        /// NPC that received the gossip.
        to_npc: NpcId,
        /// The gossip content.
        content: String,
    },
    /// A Tier-2 cognitive cycle completed for a location.
    Tier2Completed {
        /// The location whose NPCs were processed.
        location: LocationId,
        /// Human-readable summary of what happened.
        summary: String,
    },
    /// A calendar festival has begun.
    FestivalStarted {
        /// The festival that started.
        festival: Festival,
    },
    /// The season changed (e.g. Spring → Summer).
    SeasonChanged {
        /// Previous season.
        old: Season,
        /// New season.
        new: Season,
    },
}

/// Broadcast-based event bus for world events.
///
/// Wraps a `tokio::sync::broadcast::Sender` so that any component
/// can publish events and any number of subscribers can receive them
/// independently. Lagging subscribers will skip missed events rather
/// than blocking the publisher.
pub struct EventBus {
    /// The broadcast sender; cloned internally to create receivers.
    sender: broadcast::Sender<WorldEvent>,
}

impl EventBus {
    /// Creates a new event bus with a channel capacity of 256.
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(EVENT_BUS_CAPACITY);
        Self { sender }
    }

    /// Publishes an event to all current subscribers.
    ///
    /// If there are no active subscribers the event is silently dropped.
    pub fn publish(&self, event: WorldEvent) {
        // Ignore SendError (no active receivers).
        let _ = self.sender.send(event);
    }

    /// Creates a new subscription that will receive future events.
    pub fn subscribe(&self) -> broadcast::Receiver<WorldEvent> {
        self.sender.subscribe()
    }

    /// Returns the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(WorldEvent::WeatherChanged {
            old: Weather::Clear,
            new: Weather::Rain,
        });

        let event = rx.recv().await.unwrap();
        match event {
            WorldEvent::WeatherChanged { old, new } => {
                assert_eq!(old, Weather::Clear);
                assert_eq!(new, Weather::Rain);
            }
            _ => panic!("expected WeatherChanged"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(WorldEvent::SeasonChanged {
            old: Season::Spring,
            new: Season::Summer,
        });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();

        assert!(matches!(e1, WorldEvent::SeasonChanged { .. }));
        assert!(matches!(e2, WorldEvent::SeasonChanged { .. }));
    }

    #[test]
    fn test_publish_without_subscribers() {
        let bus = EventBus::new();
        // Must not panic even with zero subscribers.
        bus.publish(WorldEvent::FestivalStarted {
            festival: Festival::Bealtaine,
        });
    }

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_world_event_debug() {
        // Verify all variants implement Debug by formatting them.
        let events: Vec<WorldEvent> = vec![
            WorldEvent::WeatherChanged {
                old: Weather::Clear,
                new: Weather::Fog,
            },
            WorldEvent::NpcMoodChanged {
                npc_id: NpcId(1),
                old_mood: "content".to_string(),
                new_mood: "anxious".to_string(),
            },
            WorldEvent::NpcMoved {
                npc_id: NpcId(2),
                from: LocationId(1),
                to: LocationId(3),
            },
            WorldEvent::GossipSpread {
                from_npc: NpcId(1),
                to_npc: NpcId(2),
                content: "Did you hear about the fair?".to_string(),
            },
            WorldEvent::Tier2Completed {
                location: LocationId(5),
                summary: "NPCs exchanged greetings".to_string(),
            },
            WorldEvent::FestivalStarted {
                festival: Festival::Samhain,
            },
            WorldEvent::SeasonChanged {
                old: Season::Autumn,
                new: Season::Winter,
            },
        ];

        for event in &events {
            let debug_str = format!("{:?}", event);
            assert!(!debug_str.is_empty());
        }
    }
}
