<!--
  BansheeChill — cold blue-violet glow pulsing at screen edges.
  Something watching from just outside the frame.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="banshee" class:visible></div>

<style>
	.banshee {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 3s ease-in;
		background: radial-gradient(
			ellipse at 50% 50%,
			transparent 40%,
			rgba(80, 60, 160, 0.08) 70%,
			rgba(60, 40, 140, 0.15) 100%
		);
		animation: banshee-breathe 8s ease-in-out infinite;
	}
	.banshee.visible { opacity: 1; }

	@keyframes banshee-breathe {
		0%, 100% { opacity: 0.5; }
		40%      { opacity: 1; }
		60%      { opacity: 0.8; }
	}
</style>
