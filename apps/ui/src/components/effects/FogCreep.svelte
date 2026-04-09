<!--
  FogCreep — translucent white haze that thickens at the window edges.
  Creeps in slowly over 4s, pulses gently while active.
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

<div class="fog" class:visible></div>

<style>
	.fog {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 4s ease-in;
		background:
			radial-gradient(ellipse at 50% 50%, transparent 30%, rgba(255, 255, 255, 0.12) 70%, rgba(255, 255, 255, 0.22) 100%);
		animation: fog-breathe 8s ease-in-out infinite;
	}

	.fog.visible {
		opacity: 1;
	}

	@keyframes fog-breathe {
		0%, 100% { filter: blur(0px); opacity: 1; }
		50%      { filter: blur(1px); opacity: 0.85; }
	}
</style>
