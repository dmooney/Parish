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
	// Individual effects will be added here by subsequent commits.
];
