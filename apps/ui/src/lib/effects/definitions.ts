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
];
