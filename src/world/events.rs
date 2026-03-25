//! World event bus for cross-tier NPC communication.
//!
//! Uses `tokio::sync::broadcast` to allow any subsystem to publish
//! events and any number of subscribers to receive them.

use crate::npc::NpcId;
use crate::world::LocationId;

/// A world event published on the event bus.
#[derive(Debug, Clone)]
pub enum WorldEvent {
    /// Weather changed to a new state.
    WeatherChanged {
        /// The new weather condition.
        new_weather: String,
    },
    /// An NPC's mood changed.
    NpcMoodChanged {
        /// The NPC whose mood changed.
        npc_id: NpcId,
        /// The new mood descriptor.
        new_mood: String,
    },
    /// An NPC moved to a new location.
    NpcMoved {
        /// The NPC who moved.
        npc_id: NpcId,
        /// The destination location.
        to: LocationId,
    },
    /// Gossip was spread.
    GossipSpread {
        /// The gossip content.
        content: String,
        /// The NPC who spread the gossip.
        source: NpcId,
    },
    /// Tier 2 simulation completed for a location.
    Tier2Completed {
        /// The location where simulation ran.
        location: LocationId,
        /// A brief summary of what happened.
        summary: String,
    },
    /// A festival has started.
    FestivalStarted {
        /// The name of the festival.
        name: String,
    },
    /// Season changed.
    SeasonChanged {
        /// The new season descriptor.
        new_season: String,
    },
}

/// Event bus for broadcasting world events.
///
/// Wraps a `tokio::sync::broadcast::Sender` with a fixed capacity.
pub struct EventBus {
    sender: tokio::sync::broadcast::Sender<WorldEvent>,
}

impl EventBus {
    /// Creates a new event bus with capacity 256.
    pub fn new() -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(256);
        Self { sender }
    }

    /// Publishes an event to all subscribers.
    pub fn publish(&self, event: WorldEvent) {
        // Ignore error (no subscribers)
        let _ = self.sender.send(event);
    }

    /// Returns a new subscriber receiver.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<WorldEvent> {
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

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        bus.publish(WorldEvent::WeatherChanged {
            new_weather: "Rain".to_string(),
        });

        let event = rx.recv().await.unwrap();
        match event {
            WorldEvent::WeatherChanged { new_weather } => {
                assert_eq!(new_weather, "Rain");
            }
            _ => panic!("Expected WeatherChanged event"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        bus.publish(WorldEvent::FestivalStarted {
            name: "Samhain".to_string(),
        });

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();

        match (e1, e2) {
            (
                WorldEvent::FestivalStarted { name: n1 },
                WorldEvent::FestivalStarted { name: n2 },
            ) => {
                assert_eq!(n1, "Samhain");
                assert_eq!(n2, "Samhain");
            }
            _ => panic!("Both subscribers should receive FestivalStarted"),
        }
    }

    #[test]
    fn test_publish_without_subscribers() {
        let bus = EventBus::new();
        // Should not panic even with no subscribers.
        bus.publish(WorldEvent::SeasonChanged {
            new_season: "Winter".to_string(),
        });
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_world_event_debug() {
        let event = WorldEvent::NpcMoved {
            npc_id: NpcId(42),
            to: LocationId(7),
        };
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("NpcMoved"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("7"));
    }
}
