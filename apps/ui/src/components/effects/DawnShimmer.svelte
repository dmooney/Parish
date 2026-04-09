<!--
  DawnShimmer — brief sparkles across the screen at dawn, like dew
  catching first light. Faint, quick, gone. The world waking up.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const sparkles = Array.from({ length: 12 + Math.floor(Math.random() * 8) }, () => ({
		x: 5 + Math.random() * 90,
		y: 5 + Math.random() * 90,
		size: 1 + Math.random() * 2,
		delay: Math.random() * 6,
		duration: 1.5 + Math.random() * 2,
	}));
</script>

<div class="dawn">
	{#each sparkles as s}
		<div
			class="sparkle"
			style:left="{s.x}%"
			style:top="{s.y}%"
			style:width="{s.size}px"
			style:height="{s.size}px"
			style:--delay="{s.delay}s"
			style:--dur="{s.duration}s"
		></div>
	{/each}
</div>

<style>
	.dawn {
		position: fixed;
		inset: 0;
		pointer-events: none;
	}

	.sparkle {
		position: absolute;
		border-radius: 50%;
		background: rgba(255, 250, 220, 0.8);
		box-shadow: 0 0 3px rgba(255, 240, 180, 0.5);
		animation: sparkle-flash var(--dur) ease-in-out var(--delay) forwards;
		opacity: 0;
	}

	@keyframes sparkle-flash {
		0%   { opacity: 0; transform: scale(0.3); }
		30%  { opacity: 0.8; transform: scale(1); }
		100% { opacity: 0; transform: scale(0.5); }
	}
</style>
