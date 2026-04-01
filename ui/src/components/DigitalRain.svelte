<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { DebugSnapshot } from '$lib/types';

	export let snapshot: DebugSnapshot;

	let canvas: HTMLCanvasElement;
	let container: HTMLDivElement;
	let animFrame: number;
	let resizeObserver: ResizeObserver;

	// Column state
	interface Drop {
		x: number;
		y: number;
		speed: number;
		chars: string[];
		charIndex: number;
		brightness: number;
		source: string; // which data source this drop comes from
	}

	let drops: Drop[] = [];
	let lastTime = 0;

	const FONT_SIZE = 12;
	const CHAR_WIDTH = 8;
	const MIN_SPEED = 30;
	const MAX_SPEED = 120;
	const TRAIL_LENGTH = 18;
	const SPAWN_RATE = 0.03; // chance per column per frame to spawn a new drop

	// Collect strings from the live snapshot to use as rain content
	function harvestStrings(snap: DebugSnapshot): Map<string, string[]> {
		const sources = new Map<string, string[]>();

		// NPC names
		const names = snap.npcs.map((n) => n.name);
		if (names.length) sources.set('npc', names);

		// Location names
		const locs = snap.world.locations.map((l) => l.name);
		if (locs.length) sources.set('location', locs);

		// Moods
		const moods = [...new Set(snap.npcs.map((n) => n.mood))];
		if (moods.length) sources.set('mood', moods);

		// Events (most recent messages)
		const events = snap.events.slice(-20).map((e) => e.message);
		if (events.length) sources.set('event', events);

		// Event categories
		const cats = [...new Set(snap.events.map((e) => e.category))];
		if (cats.length) sources.set('category', cats);

		// Time data
		sources.set('time', [
			snap.clock.game_time,
			snap.clock.time_of_day,
			snap.clock.season,
			snap.clock.weather,
			snap.clock.festival ?? '',
		].filter(Boolean));

		// Occupations
		const occs = [...new Set(snap.npcs.map((n) => n.occupation))];
		if (occs.length) sources.set('occupation', occs);

		// Tier labels
		sources.set('tier', ['T1', 'T2', 'T3', 'T4', 'TIER', 'COGNITION']);

		// Inference model names
		const models = [snap.inference.provider_name, snap.inference.model_name].filter(Boolean);
		if (models.length) sources.set('inference', models);

		// Irish / Celtic flavor
		sources.set('celtic', [
			'FAILTE', 'SLAINTE', 'CRAIC', 'PAROISTE', 'BOTHARIN',
			'CLUAIN', 'DIA DUIT', 'SEAN', 'BAILE', 'ABHAINN',
		]);

		// Debug-themed decorative
		sources.set('debug', [
			'DEBUG', 'TRACE', 'TICK', 'SPAWN', 'QUEUE', 'SYNC',
			'EMIT', 'POLL', 'WAKE', 'LOCK', 'DROP', 'INIT',
		]);

		return sources;
	}

	function pickRandom<T>(arr: T[]): T {
		return arr[Math.floor(Math.random() * arr.length)];
	}

	// Convert a string to vertical character array
	function toChars(str: string): string[] {
		return str.toUpperCase().split('');
	}

	// Source category -> color tint (returned as [r,g,b])
	function sourceColor(source: string): [number, number, number] {
		switch (source) {
			case 'npc':        return [80, 255, 120];  // bright green
			case 'location':   return [120, 220, 255]; // cyan
			case 'mood':       return [255, 200, 80];  // gold
			case 'event':      return [200, 255, 100]; // lime
			case 'category':   return [180, 140, 255]; // purple
			case 'time':       return [255, 160, 100]; // orange
			case 'occupation': return [100, 255, 200]; // teal
			case 'tier':       return [255, 100, 100]; // red
			case 'inference':  return [255, 100, 255]; // magenta
			case 'celtic':     return [100, 255, 160]; // emerald
			case 'debug':      return [0, 255, 70];    // classic matrix green
			default:           return [0, 255, 70];
		}
	}

	function spawnDrop(colX: number, allStrings: Map<string, string[]>): Drop {
		const sourceKeys = [...allStrings.keys()];
		const source = pickRandom(sourceKeys);
		const pool = allStrings.get(source)!;
		const text = pickRandom(pool);

		return {
			x: colX,
			y: -FONT_SIZE * Math.random() * 10,
			speed: MIN_SPEED + Math.random() * (MAX_SPEED - MIN_SPEED),
			chars: toChars(text),
			charIndex: 0,
			brightness: 0.6 + Math.random() * 0.4,
			source,
		};
	}

	function render(time: number) {
		if (!canvas) return;
		const ctx = canvas.getContext('2d');
		if (!ctx) return;

		const dt = lastTime ? (time - lastTime) / 1000 : 0.016;
		lastTime = time;

		const w = canvas.width;
		const h = canvas.height;

		// Fade previous frame (creates trail effect)
		ctx.fillStyle = 'rgba(0, 0, 0, 0.08)';
		ctx.fillRect(0, 0, w, h);

		const allStrings = harvestStrings(snapshot);
		const cols = Math.floor(w / CHAR_WIDTH);

		// Spawn new drops
		for (let c = 0; c < cols; c++) {
			if (Math.random() < SPAWN_RATE * dt * 60) {
				drops.push(spawnDrop(c * CHAR_WIDTH, allStrings));
			}
		}

		ctx.font = `${FONT_SIZE}px monospace`;
		ctx.textBaseline = 'top';

		// Update and draw each drop
		for (let i = drops.length - 1; i >= 0; i--) {
			const drop = drops[i];
			drop.y += drop.speed * dt;

			// Which character to show at the head
			const headCharIdx = Math.floor(drop.y / FONT_SIZE) % drop.chars.length;
			const headChar = drop.chars[headCharIdx];

			const [r, g, b] = sourceColor(drop.source);

			// Draw trail
			for (let t = 0; t < TRAIL_LENGTH; t++) {
				const trailY = drop.y - t * FONT_SIZE;
				if (trailY < -FONT_SIZE || trailY > h) continue;

				const charIdx = Math.abs(Math.floor(trailY / FONT_SIZE)) % drop.chars.length;
				const ch = drop.chars[charIdx];

				if (t === 0) {
					// Head character: bright white/color
					ctx.fillStyle = `rgba(255, 255, 255, ${drop.brightness})`;
				} else if (t === 1) {
					// Second char: bright color
					ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${drop.brightness * 0.9})`;
				} else {
					// Trail fades out
					const alpha = drop.brightness * (1 - t / TRAIL_LENGTH) * 0.7;
					ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${alpha})`;
				}

				ctx.fillText(ch, drop.x, trailY);
			}

			// Remove drops that have fully scrolled off
			if (drop.y - TRAIL_LENGTH * FONT_SIZE > h) {
				drops.splice(i, 1);
			}
		}

		// Overlay: show a translucent data readout in the corner
		drawOverlay(ctx, w, h);

		animFrame = requestAnimationFrame(render);
	}

	function drawOverlay(ctx: CanvasRenderingContext2D, w: number, h: number) {
		const snap = snapshot;
		if (!snap) return;

		ctx.save();
		ctx.font = '10px monospace';
		ctx.textBaseline = 'top';

		const lines = [
			`PARISH DEBUG MATRIX`,
			`${snap.clock.game_time} | ${snap.clock.time_of_day}`,
			`${snap.clock.season} | ${snap.clock.weather}`,
			`NPCs: ${snap.npcs.length} | Locs: ${snap.world.location_count}`,
			`T1:${snap.tier_summary.tier1_count} T2:${snap.tier_summary.tier2_count} T3:${snap.tier_summary.tier3_count} T4:${snap.tier_summary.tier4_count}`,
			`Events: ${snap.events.length} | Calls: ${snap.inference.call_log.length}`,
		];

		const padding = 8;
		const lineHeight = 13;
		const boxW = 220;
		const boxH = lines.length * lineHeight + padding * 2;
		const boxX = w - boxW - 10;
		const boxY = 10;

		// Semi-transparent background
		ctx.fillStyle = 'rgba(0, 0, 0, 0.6)';
		ctx.fillRect(boxX, boxY, boxW, boxH);
		ctx.strokeStyle = 'rgba(0, 255, 70, 0.3)';
		ctx.strokeRect(boxX, boxY, boxW, boxH);

		lines.forEach((line, i) => {
			ctx.fillStyle = i === 0 ? 'rgba(0, 255, 70, 0.9)' : 'rgba(0, 255, 70, 0.6)';
			ctx.fillText(line, boxX + padding, boxY + padding + i * lineHeight);
		});

		ctx.restore();
	}

	function handleResize() {
		if (!canvas || !container) return;
		const rect = container.getBoundingClientRect();
		canvas.width = rect.width;
		canvas.height = rect.height;
	}

	onMount(() => {
		handleResize();
		resizeObserver = new ResizeObserver(handleResize);
		resizeObserver.observe(container);
		animFrame = requestAnimationFrame(render);
	});

	onDestroy(() => {
		if (animFrame) cancelAnimationFrame(animFrame);
		if (resizeObserver) resizeObserver.disconnect();
	});
</script>

<div class="rain-container" bind:this={container}>
	<canvas bind:this={canvas}></canvas>
</div>

<style>
	.rain-container {
		width: 100%;
		height: 100%;
		background: #000;
		overflow: hidden;
		position: relative;
	}

	canvas {
		display: block;
		width: 100%;
		height: 100%;
	}
</style>
