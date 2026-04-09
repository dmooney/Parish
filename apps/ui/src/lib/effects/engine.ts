/**
 * Visual effects engine — evaluates conditions and schedules effects.
 *
 * The engine runs a tick loop (every ~2s) that:
 * 1. Reads the current world context from the game store
 * 2. Checks each registered effect's conditions and cooldowns
 * 3. Rolls dice based on probability
 * 4. Activates effects by writing to the activeEffects store
 * 5. Cleans up expired effects
 *
 * Respects `prefers-reduced-motion` and an explicit user toggle.
 */

import type { EffectDefinition, EffectConditions, EffectContext, ActiveEffect } from './types';

const TICK_INTERVAL_MS = 2000;

let instanceCounter = 0;

function generateInstanceKey(effectId: string): string {
	return `${effectId}-${++instanceCounter}`;
}

/** Tests whether the current world context satisfies an effect's conditions. */
export function matchesConditions(ctx: EffectContext, cond: EffectConditions): boolean {
	if (cond.weather && !cond.weather.includes(ctx.weather)) return false;
	if (cond.season && !cond.season.includes(ctx.season)) return false;
	if (cond.timeOfDay && !cond.timeOfDay.includes(ctx.timeOfDay)) return false;
	if (cond.indoor !== undefined && cond.indoor !== ctx.indoor) return false;
	if (cond.festival) {
		if (!ctx.festival || !cond.festival.includes(ctx.festival)) return false;
	}
	if (cond.locationMatch) {
		const loc = ctx.locationName.toLowerCase();
		if (!cond.locationMatch.some((pat) => loc.includes(pat.toLowerCase()))) return false;
	}
	return true;
}

export class EffectsEngine {
	private definitions: EffectDefinition[] = [];
	private activeEffects: ActiveEffect[] = [];
	private lastActivation: Map<string, number> = new Map();
	private nextEligible: Map<string, number> = new Map();
	private tickHandle: ReturnType<typeof setInterval> | null = null;
	private enabled = true;
	private reducedMotion = false;

	/** Callbacks the engine invokes to communicate with stores. */
	private onUpdate: (effects: ActiveEffect[]) => void;
	private getContext: () => EffectContext | null;

	constructor(opts: {
		onUpdate: (effects: ActiveEffect[]) => void;
		getContext: () => EffectContext | null;
	}) {
		this.onUpdate = opts.onUpdate;
		this.getContext = opts.getContext;

		// Respect prefers-reduced-motion
		if (typeof window !== 'undefined') {
			const mq = window.matchMedia('(prefers-reduced-motion: reduce)');
			this.reducedMotion = mq.matches;
			mq.addEventListener('change', (e) => {
				this.reducedMotion = e.matches;
				if (this.reducedMotion) this.clearAll();
			});
		}
	}

	/** Registers an effect definition. */
	register(def: EffectDefinition): void {
		this.definitions.push(def);
	}

	/** Registers multiple effect definitions. */
	registerAll(defs: EffectDefinition[]): void {
		this.definitions.push(...defs);
	}

	/** Starts the tick loop. */
	start(): void {
		if (this.tickHandle) return;
		this.tickHandle = setInterval(() => this.tick(), TICK_INTERVAL_MS);
	}

	/** Stops the tick loop and clears active effects. */
	stop(): void {
		if (this.tickHandle) {
			clearInterval(this.tickHandle);
			this.tickHandle = null;
		}
		this.clearAll();
	}

	/** Enables or disables the engine (user toggle). */
	setEnabled(enabled: boolean): void {
		this.enabled = enabled;
		if (!enabled) this.clearAll();
	}

	/** Returns a copy of the current active effects. */
	getActiveEffects(): ActiveEffect[] {
		return [...this.activeEffects];
	}

	/** Manually trigger an effect by ID (for testing or one-off events). */
	trigger(effectId: string, data: Record<string, unknown> = {}): void {
		const def = this.definitions.find((d) => d.id === effectId);
		if (!def) return;
		this.activate(def, data);
	}

	private tick(): void {
		if (!this.enabled || this.reducedMotion) return;

		const now = performance.now();
		const ctx = this.getContext();
		if (!ctx) return;

		// Clean up expired effects
		this.cleanupExpired(now);

		// Evaluate each registered effect
		for (const def of this.definitions) {
			// Respect per-effect random interval scheduling
			const nextTime = this.nextEligible.get(def.id) ?? 0;
			if (now < nextTime) continue;

			// Check conditions
			if (!matchesConditions(ctx, def.conditions)) {
				// Schedule next check at a shorter interval when conditions don't match
				this.nextEligible.set(def.id, now + TICK_INTERVAL_MS);
				continue;
			}

			// Check cooldown
			const lastTime = this.lastActivation.get(def.id) ?? 0;
			if (now - lastTime < def.cooldownMs) continue;

			// Singleton check
			if (def.singleton !== false && this.activeEffects.some((e) => e.id === def.id)) {
				continue;
			}

			// Roll the dice
			if (Math.random() > def.probability) {
				// Failed the roll — schedule next attempt within the interval range
				const [minMs, maxMs] = def.intervalMs;
				const delay = minMs + Math.random() * (maxMs - minMs);
				this.nextEligible.set(def.id, now + delay);
				continue;
			}

			// Activate!
			this.activate(def, {});
		}
	}

	private activate(def: EffectDefinition, data: Record<string, unknown>): void {
		const now = performance.now();
		const effect: ActiveEffect = {
			id: def.id,
			instanceKey: generateInstanceKey(def.id),
			startedAt: now,
			durationMs: def.durationMs,
			data
		};
		this.activeEffects.push(effect);
		this.lastActivation.set(def.id, now);

		// Schedule next eligibility after the interval
		const [minMs, maxMs] = def.intervalMs;
		const delay = minMs + Math.random() * (maxMs - minMs);
		this.nextEligible.set(def.id, now + delay);

		this.onUpdate(this.getActiveEffects());

		// Schedule removal
		setTimeout(() => {
			this.activeEffects = this.activeEffects.filter((e) => e.instanceKey !== effect.instanceKey);
			this.onUpdate(this.getActiveEffects());
		}, def.durationMs);
	}

	private cleanupExpired(now: number): void {
		const before = this.activeEffects.length;
		this.activeEffects = this.activeEffects.filter(
			(e) => now - e.startedAt < e.durationMs
		);
		if (this.activeEffects.length !== before) {
			this.onUpdate(this.getActiveEffects());
		}
	}

	private clearAll(): void {
		this.activeEffects = [];
		this.onUpdate([]);
	}
}
