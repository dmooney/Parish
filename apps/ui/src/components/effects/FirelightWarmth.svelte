<!--
  FirelightWarmth — a warm amber glow that pulses irregularly indoors,
  like firelight flickering on a wall. The comfort of being inside
  on a wet night should be felt.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props {
		effect: ActiveEffect;
	}
	let { effect }: Props = $props();

	let visible = $state(false);

	onMount(() => {
		requestAnimationFrame(() => { visible = true; });
	});
</script>

<div class="firelight" class:visible></div>

<style>
	.firelight {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 3s ease-in;
		background: radial-gradient(
			ellipse at 50% 100%,
			rgba(200, 140, 50, 0.06) 0%,
			rgba(180, 120, 40, 0.03) 40%,
			transparent 70%
		);
		animation: firelight-flicker 4s ease-in-out infinite;
	}

	.firelight.visible {
		opacity: 1;
	}

	/* Irregular breathing rhythm — not a metronome but a living fire */
	@keyframes firelight-flicker {
		0%   { opacity: 1; }
		15%  { opacity: 0.7; }
		30%  { opacity: 0.95; }
		45%  { opacity: 0.6; }
		55%  { opacity: 0.85; }
		70%  { opacity: 0.75; }
		85%  { opacity: 1; }
		100% { opacity: 0.9; }
	}
</style>
