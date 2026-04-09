/**
 * Visual effects system type definitions.
 *
 * Effects are purely frontend-driven — the engine reads from the worldState
 * store (weather, season, time, location, festival) and decides which
 * ambient effects to display. No backend changes required.
 */

/** Conditions that must be met for an effect to be eligible. All specified
 *  fields must match (logical AND). Omitted fields are unconstrained. */
export interface EffectConditions {
	/** Weather states that allow this effect (e.g. ['Storm', 'HeavyRain']). */
	weather?: string[];
	/** Seasons that allow this effect (e.g. ['Autumn', 'Winter']). */
	season?: string[];
	/** Time-of-day labels that allow this effect (e.g. ['Night', 'Midnight']). */
	timeOfDay?: string[];
	/** If set, effect only plays indoors (true) or outdoors (false). */
	indoor?: boolean;
	/** Location name substrings that allow this effect (case-insensitive). */
	locationMatch?: string[];
	/** Festival names that allow this effect. */
	festival?: string[];
}

/** Static definition of a visual effect, registered with the engine. */
export interface EffectDefinition {
	/** Unique identifier (e.g. 'lightning-flash', 'fairy-sprite'). */
	id: string;
	/** Conditions for when this effect can play. */
	conditions: EffectConditions;
	/** Minimum milliseconds between activations of this effect. */
	cooldownMs: number;
	/** Random interval range [min, max] in ms between eligibility checks. */
	intervalMs: [number, number];
	/** How long the effect stays active (ms). */
	durationMs: number;
	/** Probability (0–1) of activating when eligible. Lower = rarer. */
	probability: number;
	/** If true, only one instance of this effect can be active at a time. */
	singleton?: boolean;
}

/** A currently-active effect instance. */
export interface ActiveEffect {
	/** The effect definition ID. */
	id: string;
	/** Unique instance key (for Svelte keyed each blocks). */
	instanceKey: string;
	/** Timestamp when this instance was activated (performance.now()). */
	startedAt: number;
	/** How long this instance should remain active (ms). */
	durationMs: number;
	/** Arbitrary data the renderer can use (e.g. position, target word). */
	data: Record<string, unknown>;
}

/** World context snapshot used by the engine to evaluate conditions. */
export interface EffectContext {
	weather: string;
	season: string;
	timeOfDay: string;
	hour: number;
	locationName: string;
	indoor: boolean;
	festival: string | null;
}
