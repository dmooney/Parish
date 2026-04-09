<!--
  LughnasaGold — golden chaff motes drifting in warm afternoon light.
  The harvest. Slow, lazy, warm.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });

	const motes = Array.from({ length: 10 + Math.floor(Math.random() * 8) }, () => ({
		x: 5 + Math.random() * 90,
		y: 10 + Math.random() * 80,
		size: 1 + Math.random() * 1.5,
		duration: 8 + Math.random() * 8,
		delay: Math.random() * 6,
		driftX: -10 + Math.random() * 20,
		driftY: -5 + Math.random() * 10,
	}));
</script>

<div class="lughnasa" class:visible>
	{#each motes as m}
		<div
			class="chaff"
			style:left="{m.x}%"
			style:top="{m.y}%"
			style:width="{m.size}px"
			style:height="{m.size}px"
			style:--dx="{m.driftX}px"
			style:--dy="{m.driftY}px"
			style:--dur="{m.duration}s"
			style:--delay="{m.delay}s"
		></div>
	{/each}
</div>

<style>
	.lughnasa {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 3s ease-in;
	}
	.lughnasa.visible { opacity: 1; }

	.chaff {
		position: absolute;
		border-radius: 50%;
		background: rgba(220, 190, 80, 0.6);
		animation:
			chaff-drift var(--dur) ease-in-out var(--delay) infinite alternate,
			chaff-twinkle var(--dur) ease-in-out var(--delay) infinite;
	}

	@keyframes chaff-drift {
		0%   { transform: translate(0, 0); }
		50%  { transform: translate(var(--dx), var(--dy)); }
		100% { transform: translate(calc(var(--dx) * -0.5), calc(var(--dy) * -0.5)); }
	}

	@keyframes chaff-twinkle {
		0%, 100% { opacity: 0.5; }
		30%      { opacity: 0.2; }
		60%      { opacity: 0.7; }
		80%      { opacity: 0.3; }
	}
</style>
