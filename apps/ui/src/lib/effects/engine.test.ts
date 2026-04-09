import { describe, it, expect } from 'vitest';
import { matchesConditions } from './engine';
import type { EffectContext, EffectConditions } from './types';

const baseCtx: EffectContext = {
	weather: 'Storm',
	season: 'Winter',
	timeOfDay: 'Night',
	hour: 22,
	locationName: 'The Crossroads',
	indoor: false,
	festival: null
};

describe('matchesConditions', () => {
	it('matches when no conditions are specified', () => {
		expect(matchesConditions(baseCtx, {})).toBe(true);
	});

	it('matches weather condition', () => {
		expect(matchesConditions(baseCtx, { weather: ['Storm', 'HeavyRain'] })).toBe(true);
		expect(matchesConditions(baseCtx, { weather: ['Clear'] })).toBe(false);
	});

	it('matches season condition', () => {
		expect(matchesConditions(baseCtx, { season: ['Winter', 'Autumn'] })).toBe(true);
		expect(matchesConditions(baseCtx, { season: ['Summer'] })).toBe(false);
	});

	it('matches time of day condition', () => {
		expect(matchesConditions(baseCtx, { timeOfDay: ['Night', 'Midnight'] })).toBe(true);
		expect(matchesConditions(baseCtx, { timeOfDay: ['Morning'] })).toBe(false);
	});

	it('matches indoor condition', () => {
		expect(matchesConditions(baseCtx, { indoor: false })).toBe(true);
		expect(matchesConditions(baseCtx, { indoor: true })).toBe(false);
	});

	it('matches location substring (case-insensitive)', () => {
		expect(matchesConditions(baseCtx, { locationMatch: ['crossroads'] })).toBe(true);
		expect(matchesConditions(baseCtx, { locationMatch: ['CROSS'] })).toBe(true);
		expect(matchesConditions(baseCtx, { locationMatch: ['bog', 'crossroads'] })).toBe(true);
		expect(matchesConditions(baseCtx, { locationMatch: ['pub'] })).toBe(false);
	});

	it('matches festival condition', () => {
		const festCtx = { ...baseCtx, festival: 'Samhain' as string | null };
		expect(matchesConditions(festCtx, { festival: ['Samhain'] })).toBe(true);
		expect(matchesConditions(festCtx, { festival: ['Bealtaine'] })).toBe(false);
		expect(matchesConditions(baseCtx, { festival: ['Samhain'] })).toBe(false);
	});

	it('requires ALL conditions to match (AND logic)', () => {
		expect(
			matchesConditions(baseCtx, {
				weather: ['Storm'],
				season: ['Winter'],
				indoor: false
			})
		).toBe(true);

		expect(
			matchesConditions(baseCtx, {
				weather: ['Storm'],
				season: ['Summer'],
				indoor: false
			})
		).toBe(false);
	});
});
