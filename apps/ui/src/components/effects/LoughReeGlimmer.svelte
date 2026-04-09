<!--
  LoughReeGlimmer — rippling caustic light patterns near the lake.
  Sunlight reflecting off water onto a nearby wall.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="glimmer" class:visible></div>

<style>
	.glimmer {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 3s ease-in;
		background:
			radial-gradient(ellipse 25% 15% at 30% 40%, rgba(180, 210, 230, 0.05) 0%, transparent 100%),
			radial-gradient(ellipse 20% 12% at 60% 55%, rgba(180, 210, 230, 0.04) 0%, transparent 100%),
			radial-gradient(ellipse 15% 10% at 45% 35%, rgba(200, 220, 240, 0.03) 0%, transparent 100%);
		animation: water-caustic 8s ease-in-out infinite alternate;
	}
	.glimmer.visible { opacity: 1; }

	@keyframes water-caustic {
		0% {
			background-position: 0% 0%, 0% 0%, 0% 0%;
			opacity: 0.8;
		}
		33% {
			background-position: 3% 2%, -2% 1%, 1% -1%;
			opacity: 1;
		}
		66% {
			background-position: -2% -1%, 2% -2%, -1% 2%;
			opacity: 0.7;
		}
		100% {
			background-position: 1% 1%, -1% 2%, 2% -1%;
			opacity: 0.9;
		}
	}
</style>
