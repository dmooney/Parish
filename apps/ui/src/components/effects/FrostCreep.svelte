<!--
  FrostCreep — delicate frost patterns crystallize at window corners.
  Pre-drawn SVG frost elements that fade in gradually on cold winter mornings.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });
</script>

<div class="frost" class:visible>
	<!-- Top-left corner frost -->
	<svg class="frost-corner tl" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
		<g stroke="rgba(255,255,255,0.4)" stroke-width="0.5" fill="none">
			<path d="M0 0 L40 60 L20 80 L50 120" />
			<path d="M0 0 L60 40 L80 20 L120 50" />
			<path d="M0 0 L30 30 L25 55 L45 75" />
			<path d="M20 0 L50 50 L35 70" />
			<path d="M0 20 L50 50 L70 35" />
			<path d="M40 60 L55 55 L70 70" />
			<path d="M60 40 L55 55 L70 70" />
			<circle cx="40" cy="60" r="1" fill="rgba(255,255,255,0.3)" />
			<circle cx="60" cy="40" r="1" fill="rgba(255,255,255,0.3)" />
			<circle cx="30" cy="30" r="0.8" fill="rgba(255,255,255,0.25)" />
		</g>
	</svg>

	<!-- Top-right corner frost -->
	<svg class="frost-corner tr" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
		<g stroke="rgba(255,255,255,0.35)" stroke-width="0.5" fill="none">
			<path d="M200 0 L160 60 L180 80 L150 120" />
			<path d="M200 0 L140 40 L120 20 L80 50" />
			<path d="M200 0 L170 30 L175 55 L155 75" />
			<path d="M180 0 L150 50 L165 70" />
			<path d="M200 20 L150 50 L130 35" />
			<circle cx="160" cy="60" r="1" fill="rgba(255,255,255,0.3)" />
			<circle cx="140" cy="40" r="1" fill="rgba(255,255,255,0.25)" />
		</g>
	</svg>

	<!-- Bottom-left corner frost -->
	<svg class="frost-corner bl" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
		<g stroke="rgba(255,255,255,0.3)" stroke-width="0.5" fill="none">
			<path d="M0 200 L40 140 L20 120 L50 80" />
			<path d="M0 200 L60 160 L80 180 L120 150" />
			<path d="M0 200 L30 170 L25 145" />
			<circle cx="40" cy="140" r="1" fill="rgba(255,255,255,0.25)" />
		</g>
	</svg>

	<!-- Bottom-right corner frost -->
	<svg class="frost-corner br" viewBox="0 0 200 200" xmlns="http://www.w3.org/2000/svg">
		<g stroke="rgba(255,255,255,0.3)" stroke-width="0.5" fill="none">
			<path d="M200 200 L160 140 L180 120 L150 80" />
			<path d="M200 200 L140 160 L120 180 L80 150" />
			<path d="M200 200 L170 170 L175 145" />
			<circle cx="160" cy="140" r="1" fill="rgba(255,255,255,0.25)" />
		</g>
	</svg>
</div>

<style>
	.frost {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 8s ease-in;
	}
	.frost.visible { opacity: 1; }

	.frost-corner {
		position: absolute;
		width: 200px;
		height: 200px;
	}
	.frost-corner.tl { top: 0; left: 0; }
	.frost-corner.tr { top: 0; right: 0; }
	.frost-corner.bl { bottom: 0; left: 0; }
	.frost-corner.br { bottom: 0; right: 0; }
</style>
