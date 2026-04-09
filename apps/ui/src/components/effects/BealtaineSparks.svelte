<!--
  BealtaineSparks — orange-red sparks drifting upward from below.
  From a great bonfire just out of view.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const sparks = Array.from({ length: 8 + Math.floor(Math.random() * 6) }, () => ({
		x: 20 + Math.random() * 60,
		size: 1.5 + Math.random() * 2,
		duration: 3 + Math.random() * 3,
		delay: Math.random() * 4,
		drift: -15 + Math.random() * 30,
		hue: 10 + Math.random() * 30,
	}));
</script>

<div class="sparks">
	{#each sparks as s}
		<div
			class="spark"
			style:left="{s.x}%"
			style:width="{s.size}px"
			style:height="{s.size}px"
			style:--dur="{s.duration}s"
			style:--delay="{s.delay}s"
			style:--drift="{s.drift}px"
			style:--hue="{s.hue}"
		></div>
	{/each}
</div>

<style>
	.sparks {
		position: fixed;
		inset: 0;
		pointer-events: none;
	}

	.spark {
		position: absolute;
		bottom: -5px;
		border-radius: 50%;
		background: hsl(var(--hue), 90%, 55%);
		box-shadow: 0 0 3px 1px hsla(var(--hue), 90%, 55%, 0.5);
		animation: spark-rise var(--dur) ease-out var(--delay) infinite;
		opacity: 0;
	}

	@keyframes spark-rise {
		0%   { transform: translateY(0) translateX(0); opacity: 0; }
		10%  { opacity: 0.9; }
		50%  { opacity: 0.6; background: hsl(calc(var(--hue) - 10), 70%, 40%); }
		100% { transform: translateY(-60vh) translateX(var(--drift)); opacity: 0; }
	}
</style>
