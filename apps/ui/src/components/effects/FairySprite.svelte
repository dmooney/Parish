<!--
  FairySprite — a tiny glowing golden dot that flies in from off-screen,
  lands on the text area, pulses gently, then floats away.
  Appears rarely near fairy forts, crossroads, and the bog.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props {
		effect: ActiveEffect;
	}
	let { effect }: Props = $props();

	let phase = $state<'enter' | 'rest' | 'exit'>('enter');

	// Random landing position (central area of the window)
	const landX = 15 + Math.random() * 70; // 15-85% from left
	const landY = 20 + Math.random() * 50; // 20-70% from top

	// Entry from a random edge
	const edge = Math.random();
	const startX = edge < 0.5 ? (edge < 0.25 ? -5 : 105) : 20 + Math.random() * 60;
	const startY = edge >= 0.5 ? (edge < 0.75 ? -5 : 105) : 20 + Math.random() * 60;

	// Exit to a different random edge
	const exitEdge = Math.random();
	const exitX = exitEdge < 0.5 ? (exitEdge < 0.25 ? -5 : 105) : 20 + Math.random() * 60;
	const exitY = exitEdge >= 0.5 ? (exitEdge < 0.75 ? -5 : 105) : 20 + Math.random() * 60;

	// Midpoints for the bezier-like flight path (enter)
	const midEnterX = startX + (landX - startX) * 0.4 + (Math.random() - 0.5) * 20;
	const midEnterY = startY + (landY - startY) * 0.4 + (Math.random() - 0.5) * 20;

	// Midpoints for exit path
	const midExitX = landX + (exitX - landX) * 0.4 + (Math.random() - 0.5) * 20;
	const midExitY = landY + (exitY - landY) * 0.4 + (Math.random() - 0.5) * 20;

	const enterDuration = 2500 + Math.random() * 1500; // 2.5-4s
	const restDuration = 15000 + Math.random() * 15000; // 15-30s
	const exitDuration = 2000 + Math.random() * 1500;  // 2-3.5s

	onMount(() => {
		const restTimer = setTimeout(() => { phase = 'rest'; }, enterDuration);
		const exitTimer = setTimeout(() => { phase = 'exit'; }, enterDuration + restDuration);
		return () => {
			clearTimeout(restTimer);
			clearTimeout(exitTimer);
		};
	});
</script>

<div
	class="fairy"
	class:entering={phase === 'enter'}
	class:resting={phase === 'rest'}
	class:exiting={phase === 'exit'}
	style:--start-x="{startX}vw"
	style:--start-y="{startY}vh"
	style:--mid-enter-x="{midEnterX}vw"
	style:--mid-enter-y="{midEnterY}vh"
	style:--land-x="{landX}vw"
	style:--land-y="{landY}vh"
	style:--mid-exit-x="{midExitX}vw"
	style:--mid-exit-y="{midExitY}vh"
	style:--exit-x="{exitX}vw"
	style:--exit-y="{exitY}vh"
	style:--enter-dur="{enterDuration}ms"
	style:--rest-dur="{restDuration}ms"
	style:--exit-dur="{exitDuration}ms"
></div>

<style>
	.fairy {
		position: fixed;
		width: 6px;
		height: 6px;
		border-radius: 50%;
		pointer-events: none;
		background: radial-gradient(circle, #fffbe6 0%, #ffd700 40%, transparent 70%);
		box-shadow:
			0 0 6px 2px rgba(255, 215, 0, 0.6),
			0 0 14px 4px rgba(255, 215, 0, 0.3),
			0 0 24px 8px rgba(255, 200, 0, 0.1);
		z-index: 410;
	}

	.fairy.entering {
		left: var(--start-x);
		top: var(--start-y);
		animation: fairy-enter var(--enter-dur) cubic-bezier(0.25, 0.1, 0.25, 1) forwards;
		opacity: 0;
	}

	.fairy.resting {
		left: var(--land-x);
		top: var(--land-y);
		animation: fairy-pulse 2.5s ease-in-out infinite;
		opacity: 1;
	}

	.fairy.exiting {
		left: var(--land-x);
		top: var(--land-y);
		animation: fairy-exit var(--exit-dur) cubic-bezier(0.25, 0.1, 0.25, 1) forwards;
		opacity: 1;
	}

	@keyframes fairy-enter {
		0% {
			transform: translate(0, 0) scale(0.5);
			opacity: 0;
		}
		20% {
			opacity: 0.8;
		}
		50% {
			transform: translate(
				calc(var(--mid-enter-x) - var(--start-x)),
				calc(var(--mid-enter-y) - var(--start-y))
			) scale(0.8);
		}
		100% {
			transform: translate(
				calc(var(--land-x) - var(--start-x)),
				calc(var(--land-y) - var(--start-y))
			) scale(1);
			opacity: 1;
		}
	}

	@keyframes fairy-pulse {
		0%, 100% {
			box-shadow:
				0 0 6px 2px rgba(255, 215, 0, 0.6),
				0 0 14px 4px rgba(255, 215, 0, 0.3);
			transform: translateY(0);
		}
		50% {
			box-shadow:
				0 0 8px 3px rgba(255, 215, 0, 0.8),
				0 0 20px 6px rgba(255, 215, 0, 0.4);
			transform: translateY(-2px);
		}
	}

	@keyframes fairy-exit {
		0% {
			transform: translate(0, 0) scale(1);
			opacity: 1;
		}
		50% {
			transform: translate(
				calc(var(--mid-exit-x) - var(--land-x)),
				calc(var(--mid-exit-y) - var(--land-y))
			) scale(0.8);
			opacity: 0.7;
		}
		100% {
			transform: translate(
				calc(var(--exit-x) - var(--land-x)),
				calc(var(--exit-y) - var(--land-y))
			) scale(0.3);
			opacity: 0;
		}
	}
</style>
