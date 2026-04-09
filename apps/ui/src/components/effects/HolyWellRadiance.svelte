<!--
  HolyWellRadiance — warm golden light emanating from below near
  sacred places. Gentle and constant, like being near something blessed.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="radiance" class:visible></div>

<style>
	.radiance {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 4s ease-in;
		background: radial-gradient(
			ellipse at 50% 110%,
			rgba(220, 190, 100, 0.10) 0%,
			rgba(220, 190, 100, 0.05) 30%,
			transparent 60%
		);
		animation: radiance-glow 10s ease-in-out infinite alternate;
	}
	.radiance.visible { opacity: 1; }

	@keyframes radiance-glow {
		0%   { opacity: 0.8; }
		100% { opacity: 1; }
	}
</style>
