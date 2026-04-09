<!--
  SamhainCandles — flickering amber dots at the screen edges.
  Candles in distant windows, guiding the dead home.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const candles = Array.from({ length: 5 + Math.floor(Math.random() * 4) }, () => {
		const edge = Math.random();
		return {
			x: edge < 0.5 ? (edge < 0.25 ? 2 + Math.random() * 5 : 93 + Math.random() * 5) : 10 + Math.random() * 80,
			y: edge >= 0.5 ? (edge < 0.75 ? 5 + Math.random() * 10 : 85 + Math.random() * 10) : 10 + Math.random() * 80,
			size: 2 + Math.random() * 2,
			duration: 1.5 + Math.random() * 2,
			delay: Math.random() * 3,
		};
	});
</script>

<div class="samhain">
	{#each candles as c}
		<div
			class="candle"
			style:left="{c.x}%"
			style:top="{c.y}%"
			style:width="{c.size}px"
			style:height="{c.size}px"
			style:--dur="{c.duration}s"
			style:--delay="{c.delay}s"
		></div>
	{/each}
</div>

<style>
	.samhain {
		position: fixed;
		inset: 0;
		pointer-events: none;
	}

	.candle {
		position: absolute;
		border-radius: 50%;
		background: #ffaa33;
		box-shadow: 0 0 4px 1px rgba(255, 170, 50, 0.6), 0 0 10px 3px rgba(255, 140, 20, 0.2);
		animation: candle-flicker var(--dur) ease-in-out var(--delay) infinite;
	}

	@keyframes candle-flicker {
		0%, 100% { opacity: 0.8; transform: scale(1); }
		25%      { opacity: 0.5; transform: scale(0.85); }
		50%      { opacity: 0.9; transform: scale(1.05); }
		75%      { opacity: 0.6; transform: scale(0.9); }
	}
</style>
