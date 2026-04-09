<!--
  VeilThins — prismatic shimmer at the screen edges at crossroads.
  The veil between worlds is thin here. A subtle chromatic aberration.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="veil" class:visible></div>

<style>
	.veil {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 4s ease-in;
		background:
			linear-gradient(135deg,
				rgba(180, 120, 255, 0.04) 0%,
				transparent 15%,
				transparent 85%,
				rgba(100, 200, 255, 0.04) 100%
			),
			linear-gradient(225deg,
				rgba(255, 180, 120, 0.03) 0%,
				transparent 15%,
				transparent 85%,
				rgba(120, 255, 180, 0.03) 100%
			);
		animation: veil-shimmer 6s ease-in-out infinite alternate;
	}
	.veil.visible { opacity: 1; }

	@keyframes veil-shimmer {
		0% {
			filter: hue-rotate(0deg);
			opacity: 1;
		}
		50% {
			filter: hue-rotate(15deg);
			opacity: 0.6;
		}
		100% {
			filter: hue-rotate(-10deg);
			opacity: 1;
		}
	}
</style>
