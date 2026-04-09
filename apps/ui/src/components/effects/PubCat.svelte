<!--
  PubCat — a tiny cat silhouette walking across the bottom of the screen.
  Padding left to right, tail up. Darcy's Pub has a cat.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const fromRight = Math.random() > 0.5;
	const duration = 7 + Math.random() * 3;
</script>

<div
	class="cat-wrap"
	class:from-right={fromRight}
	style:--dur="{duration}s"
>
	<svg
		class="cat"
		width="22" height="16" viewBox="0 0 22 16"
		class:flipped={fromRight}
	>
		<g fill="var(--color-fg, #31240f)" opacity="0.6">
			<!-- Body -->
			<ellipse cx="11" cy="11" rx="5" ry="3" />
			<!-- Head -->
			<circle cx="17" cy="9" r="2.2" />
			<!-- Ears -->
			<polygon points="15.5,7 16,4.5 17,6.5" />
			<polygon points="17.5,6.5 18.5,4 19,7" />
			<!-- Tail (curved up) -->
			<path d="M5,10 Q2,8 3,4" stroke="var(--color-fg, #31240f)" stroke-width="1" fill="none" />
			<!-- Legs -->
			<rect x="8" y="13" width="1" height="3" rx="0.5" />
			<rect x="13" y="13" width="1" height="3" rx="0.5" />
		</g>
	</svg>
</div>

<style>
	.cat-wrap {
		position: fixed;
		bottom: 4px;
		left: -30px;
		pointer-events: none;
		z-index: 410;
		animation: cat-walk var(--dur) linear forwards;
	}

	.cat-wrap.from-right {
		left: auto;
		right: -30px;
		animation-name: cat-walk-right;
	}

	.cat {
		animation: cat-bob 0.5s ease-in-out infinite;
	}
	.cat.flipped { transform: scaleX(-1); }

	@keyframes cat-walk {
		0%   { transform: translateX(0); }
		100% { transform: translateX(calc(100vw + 60px)); }
	}

	@keyframes cat-walk-right {
		0%   { transform: translateX(0); }
		100% { transform: translateX(calc(-100vw - 60px)); }
	}

	@keyframes cat-bob {
		0%, 100% { transform: translateY(0); }
		50%      { transform: translateY(-1px); }
	}
</style>
