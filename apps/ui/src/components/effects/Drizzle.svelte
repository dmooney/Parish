<!--
  Drizzle — tiny water droplets that appear and slide down the window.
  Like condensation on glass during light rain.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);

	// Generate 6-10 droplets at random positions
	const drops = Array.from({ length: 6 + Math.floor(Math.random() * 5) }, () => ({
		x: 5 + Math.random() * 90,
		y: 5 + Math.random() * 40,
		size: 2 + Math.random() * 2,
		delay: Math.random() * 8,
		duration: 6 + Math.random() * 6,
		slideDistance: 30 + Math.random() * 50,
	}));

	onMount(() => {
		requestAnimationFrame(() => { visible = true; });
	});
</script>

<div class="drizzle" class:visible>
	{#each drops as drop}
		<div
			class="drop"
			style:left="{drop.x}%"
			style:top="{drop.y}%"
			style:width="{drop.size}px"
			style:height="{drop.size}px"
			style:--slide="{drop.slideDistance}px"
			style:--delay="{drop.delay}s"
			style:--dur="{drop.duration}s"
		></div>
	{/each}
</div>

<style>
	.drizzle {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 2s ease-in;
	}
	.drizzle.visible { opacity: 1; }

	.drop {
		position: absolute;
		border-radius: 50%;
		background: rgba(200, 210, 220, 0.25);
		box-shadow: 0 0 2px rgba(200, 210, 220, 0.15);
		animation: drop-slide var(--dur) ease-in var(--delay) infinite;
	}

	@keyframes drop-slide {
		0%   { transform: translateY(0); opacity: 0; }
		10%  { opacity: 0.5; }
		80%  { opacity: 0.3; }
		100% { transform: translateY(var(--slide)); opacity: 0; }
	}
</style>
