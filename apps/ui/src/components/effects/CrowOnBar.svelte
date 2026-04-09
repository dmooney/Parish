<!--
  CrowOnBar — a tiny crow silhouette that lands on the top of the screen,
  tilts its head, then flies away. Pure SVG, pure delight.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let phase = $state<'land' | 'sit' | 'fly'>('land');
	const x = 15 + Math.random() * 70;
	const sitDuration = 8000 + Math.random() * 12000;

	onMount(() => {
		const sitTimer = setTimeout(() => { phase = 'sit'; }, 800);
		const flyTimer = setTimeout(() => { phase = 'fly'; }, 800 + sitDuration);
		return () => { clearTimeout(sitTimer); clearTimeout(flyTimer); };
	});
</script>

<div class="crow-wrap" style:left="{x}%">
	<svg
		class="crow"
		class:landing={phase === 'land'}
		class:sitting={phase === 'sit'}
		class:flying={phase === 'fly'}
		width="14" height="12" viewBox="0 0 14 12"
	>
		<g fill="var(--color-fg, #31240f)" opacity="0.7">
			<!-- Body -->
			<ellipse cx="7" cy="8" rx="3" ry="2.5" />
			<!-- Head -->
			<circle cx="10" cy="6" r="1.8" />
			<!-- Beak -->
			<polygon points="12,5.5 14,6 12,6.5" />
			<!-- Tail -->
			<polygon points="3,7 1,5 2,8" />
			<!-- Legs -->
			<line x1="6" y1="10" x2="5.5" y2="12" stroke="var(--color-fg, #31240f)" stroke-width="0.5" />
			<line x1="8" y1="10" x2="8.5" y2="12" stroke="var(--color-fg, #31240f)" stroke-width="0.5" />
		</g>
	</svg>
</div>

<style>
	.crow-wrap {
		position: fixed;
		top: 0;
		pointer-events: none;
		z-index: 410;
	}

	.crow.landing {
		animation: crow-land 800ms ease-out forwards;
	}

	.crow.sitting {
		animation: crow-tilt 3s ease-in-out infinite;
	}

	.crow.flying {
		animation: crow-fly 1.2s ease-in forwards;
	}

	@keyframes crow-land {
		0%   { transform: translateY(-20px) scale(0.6); opacity: 0; }
		60%  { transform: translateY(2px) scale(1); opacity: 1; }
		100% { transform: translateY(0) scale(1); opacity: 1; }
	}

	@keyframes crow-tilt {
		0%, 100% { transform: rotate(0deg); }
		30%      { transform: rotate(8deg); }
		60%      { transform: rotate(-5deg); }
	}

	@keyframes crow-fly {
		0%   { transform: translate(0, 0) scale(1); opacity: 1; }
		100% { transform: translate(60px, -40px) scale(0.5); opacity: 0; }
	}
</style>
