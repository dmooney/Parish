<!--
  MothAtLamp — a tiny moth fluttering in erratic circles near
  the center-top of the screen, drawn to the light of your screen.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const centerX = 40 + Math.random() * 20;
	const centerY = 15 + Math.random() * 20;
	const orbitSize = 15 + Math.random() * 20;
	const duration = 3 + Math.random() * 2;
</script>

<div
	class="moth-orbit"
	style:left="{centerX}%"
	style:top="{centerY}%"
	style:--orbit="{orbitSize}px"
	style:--dur="{duration}s"
>
	<svg class="moth" width="5" height="5" viewBox="0 0 10 10">
		<g fill="var(--color-muted, #76663b)" opacity="0.5">
			<ellipse cx="5" cy="5" rx="2" ry="3" />
			<ellipse cx="3" cy="4" rx="2" ry="1.5" transform="rotate(-20 3 4)" />
			<ellipse cx="7" cy="4" rx="2" ry="1.5" transform="rotate(20 7 4)" />
		</g>
	</svg>
</div>

<style>
	.moth-orbit {
		position: fixed;
		pointer-events: none;
		z-index: 410;
		animation: moth-flutter var(--dur) ease-in-out infinite;
	}

	.moth {
		animation: moth-wings 0.15s linear infinite alternate;
	}

	@keyframes moth-flutter {
		0%   { transform: translate(0, 0); }
		15%  { transform: translate(var(--orbit), calc(var(--orbit) * -0.5)); }
		30%  { transform: translate(calc(var(--orbit) * -0.3), calc(var(--orbit) * -0.8)); }
		45%  { transform: translate(calc(var(--orbit) * -0.8), calc(var(--orbit) * 0.2)); }
		60%  { transform: translate(calc(var(--orbit) * 0.5), var(--orbit)); }
		75%  { transform: translate(calc(var(--orbit) * 0.9), calc(var(--orbit) * -0.3)); }
		90%  { transform: translate(calc(var(--orbit) * -0.4), calc(var(--orbit) * 0.6)); }
		100% { transform: translate(0, 0); }
	}

	@keyframes moth-wings {
		0%   { transform: scaleX(1); }
		100% { transform: scaleX(0.6); }
	}
</style>
