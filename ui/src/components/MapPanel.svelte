<script lang="ts">
	import { mapData } from '../stores/game';
	import { fullMapOpen } from '../stores/game';
	import { submitInput } from '$lib/ipc';
	import { resolveLabels, distSq, estimateTextWidth } from '$lib/map-labels';
	import { projectWorld, clampToRect } from '$lib/map-projection';
	import type { MapLocation } from '$lib/types';
	import type { ProjectedLocation } from '$lib/map-projection';
	import type { ResolvedLabel } from '$lib/map-labels';
	import { tweened } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';

	/** Reference dimensions — visual sizes are authored relative to this. */
	const W = 320;
	const H = 240;
	/** Base sizes at the reference scale (W × H viewBox). */
	const BASE_NODE_R = 5;
	const BASE_PLAYER_R = 8;
	const BASE_FONT_SIZE = 7;
	/** Only show locations within this many hops on the minimap. */
	const MINIMAP_HOP_RADIUS = 3;
	/** Show off-screen indicators for locations up to this many hops away. */
	const OFFSCREEN_HOP_LIMIT = 5;

	// Tweened center for smooth panning
	const viewCenter = tweened({ x: 0, y: 0 }, { duration: 400, easing: cubicOut });

	// Project ALL locations in world-space (stable coordinates)
	let allProjected: ProjectedLocation[] = $derived(
		projectWorld($mapData?.locations ?? [])
	);

	// Filter to minimap-visible locations
	let nearbyProjected: ProjectedLocation[] = $derived(
		allProjected.filter((l) => l.hops <= MINIMAP_HOP_RADIUS)
	);

	// Find the player's world-space position and update the tweened center
	let playerWorld: { x: number; y: number } | null = $derived.by(() => {
		const p = allProjected.find((l) => $mapData?.player_location === l.id);
		return p ? { x: p.x, y: p.y } : null;
	});

	// Update tweened center when player moves
	$effect(() => {
		if (playerWorld) {
			viewCenter.set({ x: playerWorld.x, y: playerWorld.y });
		}
	});

	// Compute bounding box of nearby locations relative to the player, then derive
	// a viewBox that fits them all with padding.  This auto-zooms the minimap so
	// that neighbours are always visible regardless of geographic spread.
	let viewBox: { x: number; y: number; w: number; h: number } = $derived.by(() => {
		if (nearbyProjected.length === 0) return { x: 0, y: 0, w: W, h: H };

		const cx = $viewCenter.x;
		const cy = $viewCenter.y;

		let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
		for (const l of nearbyProjected) {
			const rx = l.x - cx;
			const ry = l.y - cy;
			if (rx < minX) minX = rx;
			if (rx > maxX) maxX = rx;
			if (ry < minY) minY = ry;
			if (ry > maxY) maxY = ry;
		}

		const PAD = 40; // px padding around the bounding box
		const spanX = maxX - minX + PAD * 2;
		const spanY = maxY - minY + PAD * 2;

		// Maintain the W:H aspect ratio, using whichever axis is tighter
		const aspect = W / H;
		let vbW: number, vbH: number;
		if (spanX / spanY > aspect) {
			vbW = Math.max(spanX, 80);
			vbH = vbW / aspect;
		} else {
			vbH = Math.max(spanY, 60);
			vbW = vbH * aspect;
		}

		const midX = (minX + maxX) / 2;
		const midY = (minY + maxY) / 2;

		return { x: midX - vbW / 2, y: midY - vbH / 2, w: vbW, h: vbH };
	});

	// Scale factor: how much bigger the viewBox is compared to the reference W×H.
	// All visual sizes (radii, fonts, strokes) are multiplied by this so they
	// appear the same on screen regardless of geographic spread.
	let s: number = $derived(viewBox.w / W);
	let nodeR: number = $derived(BASE_NODE_R * s);
	let playerR: number = $derived(BASE_PLAYER_R * s);
	let fontSize: number = $derived(BASE_FONT_SIZE * s);

	// Transform nearby locations to viewBox-local coordinates (centered on player)
	let localProjected: ProjectedLocation[] = $derived(
		nearbyProjected.map((l) => ({
			...l,
			x: l.x - $viewCenter.x - viewBox.x,
			y: l.y - $viewCenter.y - viewBox.y
		}))
	);

	let labels: ResolvedLabel[] = $derived(
		resolveLabels(
			localProjected.map((loc) => ({
				nodeX: loc.x,
				nodeY: loc.y,
				nodeR: isPlayer(loc) ? playerR : nodeR,
				textW: estimateTextWidth(loc.name) * s,
				textH: fontSize
			})),
			viewBox.w,
			viewBox.h
		)
	);

	// Off-screen indicators: locations beyond minimap radius but within indicator limit
	let offscreenIndicators: { x: number; y: number; angle: number; name: string }[] = $derived.by(
		() => {
			const halfW = viewBox.w / 2;
			const halfH = viewBox.h / 2;
			const cx = viewBox.w / 2;
			const cy = viewBox.h / 2;

			return allProjected
				.filter(
					(l) => l.hops > MINIMAP_HOP_RADIUS && l.hops <= OFFSCREEN_HOP_LIMIT
				)
				.map((l) => {
					const localX = l.x - $viewCenter.x - viewBox.x;
					const localY = l.y - $viewCenter.y - viewBox.y;
					const clamped = clampToRect(localX, localY, cx, cy, halfW - 8 * s, halfH - 8 * s);
					return {
						x: clamped.x,
						y: clamped.y,
						angle: clamped.angle,
						name: l.name
					};
				});
		}
	);

	// Edges filtered to those where both endpoints are in nearbyProjected
	let visibleEdges: [string, string][] = $derived.by(() => {
		const nearbyIds = new Set(nearbyProjected.map((l) => l.id));
		return ($mapData?.edges ?? []).filter(([a, b]) => nearbyIds.has(a) && nearbyIds.has(b));
	});

	let tooltip: string | null = $state(null);

	function isPlayer(loc: MapLocation): boolean {
		return $mapData?.player_location === loc.id;
	}

	async function handleClick(loc: MapLocation) {
		if (!loc.adjacent) return;
		await submitInput(`go to ${loc.name}`);
	}

	function openFullMap() {
		fullMapOpen.set(true);
	}
</script>

<div class="map-panel" data-testid="map-panel">
	<div class="map-header">
		<span class="map-title">Map</span>
		<button class="expand-btn" onclick={openFullMap} title="Open full map (M)">
			<svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor">
				<path d="M1 1h5v2H3v3H1V1zm9 0h5v5h-2V3h-3V1zM1 10h2v3h3v2H1v-5zm12 3h-3v2h5v-5h-2v3z" />
			</svg>
		</button>
	</div>
	{#if $mapData}
		<svg viewBox="0 0 {viewBox.w} {viewBox.h}" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Parish minimap">
			<!-- Edges -->
			{#each visibleEdges as [src, dst]}
				{@const a = localProjected.find((p) => p.id === src)}
				{@const b = localProjected.find((p) => p.id === dst)}
				{#if a && b}
					<line x1={a.x} y1={a.y} x2={b.x} y2={b.y} class="edge" stroke-width={1 * s} />
				{/if}
			{/each}

			<!-- Leader lines (drawn behind labels) -->
			{#each localProjected as loc, i}
				{@const label = labels[i]}
				{@const r = isPlayer(loc) ? playerR : nodeR}
				{@const threshold = (r + 6 * s) ** 2}
				{#if label && distSq(label.cx, label.cy, label.ax, label.ay) > threshold}
					<line
						x1={loc.x}
						y1={loc.y + r + 1 * s}
						x2={label.cx}
						y2={label.cy - label.h / 2}
						class="leader"
						stroke-width={0.3 * s}
					/>
				{/if}
			{/each}

			<!-- Location nodes -->
			{#each localProjected as loc, i}
				{@const label = labels[i]}
				<!-- svelte-ignore a11y_click_events_have_key_events -->
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<g
					class="node"
					class:player={isPlayer(loc)}
					class:adjacent={loc.adjacent}
					onclick={() => handleClick(loc)}
					onmouseenter={() => (tooltip = loc.name)}
					onmouseleave={() => (tooltip = null)}
				>
					<circle cx={loc.x} cy={loc.y} r={isPlayer(loc) ? playerR : nodeR} class="node-circle" stroke-width={1.5 * s} />
					{#if label}
						<text x={label.cx} y={label.cy + fontSize / 2 - 1 * s} class="node-label" font-size={fontSize}>
							{loc.name.length > 14 ? loc.name.slice(0, 12) + '\u2026' : loc.name}
						</text>
					{/if}
				</g>
			{/each}

			<!-- Off-screen indicators -->
			{#each offscreenIndicators as ind}
				<g
					transform="translate({ind.x},{ind.y}) rotate({(ind.angle * 180) / Math.PI})"
					class="offscreen-indicator"
				>
					<polygon points="0,{-3 * s} {6 * s},0 0,{3 * s}" />
				</g>
			{/each}
		</svg>
		{#if tooltip}
			<div class="tooltip">{tooltip}</div>
		{/if}
	{:else}
		<div class="empty">Loading map&hellip;</div>
	{/if}
</div>

<style>
	.map-panel {
		background: var(--color-panel-bg);
		border-left: 1px solid var(--color-border);
		border-bottom: 1px solid var(--color-border);
		padding: 0.5rem;
		position: relative;
		flex-shrink: 0;
	}

	.map-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 0.25rem;
	}

	.map-title {
		font-size: 0.75rem;
		color: var(--color-muted);
		text-transform: uppercase;
		letter-spacing: 0.08em;
	}

	.expand-btn {
		background: none;
		border: none;
		color: var(--color-muted);
		cursor: pointer;
		padding: 2px;
		line-height: 1;
		border-radius: 2px;
	}

	.expand-btn:hover {
		color: var(--color-accent);
		background: var(--color-input-bg);
	}

	svg {
		width: 100%;
		height: auto;
		display: block;
	}

	.edge {
		stroke: var(--color-border);
	}

	.leader {
		stroke: var(--color-muted);
		stroke-dasharray: 1.5 1;
	}

	.node-circle {
		fill: var(--color-panel-bg);
		stroke: var(--color-muted);
		cursor: default;
	}

	.node.adjacent .node-circle {
		stroke: var(--color-accent);
		cursor: pointer;
	}

	.node.adjacent .node-circle:hover {
		fill: var(--color-input-bg);
	}

	.node.player .node-circle {
		fill: var(--color-accent);
		stroke: var(--color-fg);
	}

	.node-label {
		fill: var(--color-muted);
		text-anchor: middle;
		pointer-events: none;
	}

	.node.player .node-label {
		fill: var(--color-fg);
	}

	.offscreen-indicator polygon {
		fill: var(--color-muted);
		opacity: 0.6;
	}

	.tooltip {
		position: absolute;
		bottom: 0.5rem;
		right: 0.5rem;
		background: var(--color-input-bg);
		border: 1px solid var(--color-border);
		color: var(--color-fg);
		padding: 0.2rem 0.5rem;
		font-size: 0.8rem;
		border-radius: 3px;
		pointer-events: none;
	}

	.empty {
		color: var(--color-muted);
		font-style: italic;
		font-size: 0.85rem;
		text-align: center;
		padding: 2rem;
	}
</style>
