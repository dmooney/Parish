/**
 * Registry of all visual effect definitions.
 *
 * Each effect is a static definition — the engine evaluates conditions
 * and probability, the EffectsLayer renders the active instances.
 * Individual effect renderers live in components/effects/*.svelte.
 */

import type { EffectDefinition } from './types';

export const EFFECT_DEFINITIONS: EffectDefinition[] = [
	// ── Weather ──────────────────────────────────────────────────────────

	{
		id: 'lightning-flash',
		conditions: { weather: ['Storm'], indoor: false },
		cooldownMs: 15_000,
		intervalMs: [30_000, 90_000],
		durationMs: 800,
		probability: 0.7,
		singleton: true,
	},

	{
		id: 'rain-streaks',
		conditions: { weather: ['LightRain', 'HeavyRain', 'Storm'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'drizzle',
		conditions: { weather: ['LightRain'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'fog-creep',
		conditions: { weather: ['Fog'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'wind-gust',
		conditions: { weather: ['Storm', 'HeavyRain'], indoor: false },
		cooldownMs: 30_000,
		intervalMs: [20_000, 60_000],
		durationMs: 700,
		probability: 0.5,
		singleton: true,
	},

	{
		id: 'rain-ink-bleed',
		conditions: { weather: ['HeavyRain', 'Storm'] },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'frost-creep',
		conditions: {
			season: ['Winter'],
			indoor: false,
			timeOfDay: ['Dawn', 'Morning'],
			weather: ['Clear', 'PartlyCloudy', 'Overcast'],
		},
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'moonlit-text',
		conditions: {
			indoor: false,
			timeOfDay: ['Night', 'Midnight'],
			weather: ['Clear', 'PartlyCloudy'],
		},
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	// ── Folklore ─────────────────────────────────────────────────────────

	{
		id: 'fairy-sprite',
		conditions: {
			indoor: false,
			locationMatch: ['crossroads', 'fairy', 'rath', 'fort', 'bog', 'hawthorn'],
		},
		cooldownMs: 600_000,
		intervalMs: [300_000, 900_000],
		durationMs: 35_000,
		probability: 0.3,
		singleton: true,
	},

	{
		id: 'bog-lights',
		conditions: {
			indoor: false,
			locationMatch: ['bog', 'marsh', 'moor', 'turf'],
			timeOfDay: ['Dusk', 'Night', 'Midnight'],
		},
		cooldownMs: 120_000,
		intervalMs: [60_000, 180_000],
		durationMs: 45_000,
		probability: 0.5,
		singleton: true,
	},

	{
		id: 'veil-thins',
		conditions: {
			indoor: false,
			locationMatch: ['crossroads'],
			timeOfDay: ['Dusk', 'Night', 'Midnight'],
		},
		cooldownMs: 120_000,
		intervalMs: [60_000, 240_000],
		durationMs: 30_000,
		probability: 0.4,
		singleton: true,
	},

	{
		id: 'banshee-chill',
		conditions: {
			indoor: false,
			timeOfDay: ['Night', 'Midnight'],
		},
		cooldownMs: 600_000,
		intervalMs: [300_000, 900_000],
		durationMs: 15_000,
		probability: 0.15,
		singleton: true,
	},

	{
		id: 'holy-well-radiance',
		conditions: {
			indoor: false,
			locationMatch: ['well', 'church', 'brigid', 'holy'],
		},
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	// ── Living World ─────────────────────────────────────────────────────

	{
		id: 'crow-on-bar',
		conditions: { indoor: false },
		cooldownMs: 300_000,
		intervalMs: [180_000, 600_000],
		durationMs: 20_000,
		probability: 0.3,
		singleton: true,
	},

	{
		id: 'pub-cat',
		conditions: {
			indoor: true,
			locationMatch: ['pub', 'darcy'],
		},
		cooldownMs: 300_000,
		intervalMs: [300_000, 600_000],
		durationMs: 12_000,
		probability: 0.4,
		singleton: true,
	},

	{
		id: 'moth-at-lamp',
		conditions: {
			indoor: true,
			timeOfDay: ['Dusk', 'Night', 'Midnight'],
		},
		cooldownMs: 180_000,
		intervalMs: [120_000, 300_000],
		durationMs: 15_000,
		probability: 0.3,
		singleton: true,
	},

	{
		id: 'spider-thread',
		conditions: { indoor: true },
		cooldownMs: 1_800_000,
		intervalMs: [900_000, 1_800_000],
		durationMs: 14_000,
		probability: 0.2,
		singleton: true,
	},

	{
		id: 'turf-smoke',
		conditions: {
			indoor: false,
			locationMatch: ['pub', 'darcy', 'cottage', 'farm', 'murphy', 'village', 'shop'],
		},
		cooldownMs: 30_000,
		intervalMs: [20_000, 60_000],
		durationMs: 20_000,
		probability: 0.6,
		singleton: true,
	},

	// ── Ambient ──────────────────────────────────────────────────────────

	{
		id: 'firelight-warmth',
		conditions: { indoor: true },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'dust-motes',
		conditions: {
			indoor: true,
			weather: ['Clear', 'PartlyCloudy'],
			timeOfDay: ['Morning', 'Midday', 'Afternoon'],
		},
		cooldownMs: 60_000,
		intervalMs: [30_000, 120_000],
		durationMs: 30_000,
		probability: 0.5,
		singleton: true,
	},

	{
		id: 'breathing-page',
		conditions: {},
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 120_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'aurora-borealis',
		conditions: {
			indoor: false,
			season: ['Winter'],
			timeOfDay: ['Night', 'Midnight'],
			weather: ['Clear'],
		},
		cooldownMs: 600_000,
		intervalMs: [300_000, 900_000],
		durationMs: 60_000,
		probability: 0.25,
		singleton: true,
	},

	{
		id: 'dawn-shimmer',
		conditions: {
			indoor: false,
			timeOfDay: ['Dawn'],
		},
		cooldownMs: 60_000,
		intervalMs: [30_000, 90_000],
		durationMs: 10_000,
		probability: 0.6,
		singleton: true,
	},

	{
		id: 'lough-ree-glimmer',
		conditions: {
			indoor: false,
			locationMatch: ['lough', 'lake', 'shore', 'bay', 'hodson'],
			weather: ['Clear', 'PartlyCloudy'],
		},
		cooldownMs: 60_000,
		intervalMs: [30_000, 120_000],
		durationMs: 30_000,
		probability: 0.5,
		singleton: true,
	},

	// ── Seasonal ─────────────────────────────────────────────────────────

	{
		id: 'autumn-leaf',
		conditions: { season: ['Autumn'], indoor: false },
		cooldownMs: 15_000,
		intervalMs: [20_000, 60_000],
		durationMs: 9_000,
		probability: 0.5,
		singleton: true,
	},

	{
		id: 'samhain-candles',
		conditions: { festival: ['Samhain'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'bealtaine-sparks',
		conditions: { festival: ['Bealtaine'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'imbolc-thaw',
		conditions: { festival: ['Imbolc'], indoor: false },
		cooldownMs: 0,
		intervalMs: [1_000, 2_000],
		durationMs: 60_000,
		probability: 1.0,
		singleton: true,
	},

	{
		id: 'lughnasa-gold',
		conditions: {
			season: ['Summer'],
			indoor: false,
			timeOfDay: ['Afternoon'],
			weather: ['Clear', 'PartlyCloudy'],
		},
		cooldownMs: 60_000,
		intervalMs: [30_000, 120_000],
		durationMs: 30_000,
		probability: 0.5,
		singleton: true,
	},

	// ── Interaction-triggered ────────────────────────────────────────────
	// These use low cooldowns and are triggered manually by the engine
	// via the EffectsLayer rather than the standard condition system.
	// They're registered here for consistency but activated via trigger().

	{
		id: 'ink-splash',
		conditions: {},
		cooldownMs: 2_000,
		intervalMs: [999_999, 999_999],
		durationMs: 500,
		probability: 0,
		singleton: true,
	},

	{
		id: 'page-turn',
		conditions: {},
		cooldownMs: 2_000,
		intervalMs: [999_999, 999_999],
		durationMs: 600,
		probability: 0,
		singleton: true,
	},

	{
		id: 'candle-gutter',
		conditions: {
			timeOfDay: ['Night', 'Midnight'],
		},
		cooldownMs: 300_000,
		intervalMs: [999_999, 999_999],
		durationMs: 30_000,
		probability: 0,
		singleton: true,
	},
];
