<!--
  AuroraBorealis — slow undulating curtains of green and violet across
  the top third. Very faint, almost subliminal. A gift for the patient.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="aurora" class:visible></div>

<style>
	.aurora {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		height: 40%;
		pointer-events: none;
		opacity: 0;
		transition: opacity 8s ease-in;
		background: linear-gradient(
			180deg,
			rgba(80, 200, 120, 0.04) 0%,
			rgba(100, 160, 200, 0.03) 20%,
			rgba(140, 100, 200, 0.02) 40%,
			transparent 100%
		);
		animation: aurora-wave 30s ease-in-out infinite alternate;
	}
	.aurora.visible { opacity: 1; }

	@keyframes aurora-wave {
		0% {
			background-position: 0% 0%;
			filter: hue-rotate(0deg);
		}
		33% {
			filter: hue-rotate(20deg);
		}
		66% {
			filter: hue-rotate(-15deg);
		}
		100% {
			background-position: 100% 0%;
			filter: hue-rotate(10deg);
		}
	}
</style>
