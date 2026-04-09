<!--
  MoonlitText — a soft silvery patch of light drifting slowly across
  the window on clear nights, as if moonbeams are shifting through clouds.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);

	const startX = Math.random() * 60;
	const driftX = 20 + Math.random() * 30;

	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div
	class="moonlight"
	class:visible
	style:--start-x="{startX}%"
	style:--drift-x="{driftX}%"
></div>

<style>
	.moonlight {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 4s ease-in;
		background: radial-gradient(
			ellipse 40% 70% at var(--start-x) 30%,
			rgba(200, 210, 230, 0.06) 0%,
			transparent 70%
		);
		animation: moon-drift 60s linear forwards;
	}
	.moonlight.visible { opacity: 1; }

	@keyframes moon-drift {
		0%   { background-position: 0% 0%; filter: brightness(1); }
		30%  { filter: brightness(1.1); }
		50%  { filter: brightness(0.9); }
		70%  { filter: brightness(1.05); }
		100% {
			background-position: var(--drift-x) 10%;
			filter: brightness(1);
		}
	}
</style>
