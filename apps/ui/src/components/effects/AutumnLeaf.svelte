<!--
  AutumnLeaf — a single leaf drifting diagonally across the screen,
  tumbling in a realistic falling-leaf pattern. One at a time.
  Muted brown-orange-red. Takes 6-8s to cross.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props {
		effect: ActiveEffect;
	}
	let { effect }: Props = $props();

	// Leaf properties
	const colors = ['#8B4513', '#A0522D', '#CD853F', '#B8602A', '#6B3A2A', '#9E4A2F'];
	const color = colors[Math.floor(Math.random() * colors.length)];

	const startX = 10 + Math.random() * 60; // start in left-center area
	const duration = 6 + Math.random() * 2;  // 6-8s
	const drift = 20 + Math.random() * 30;   // horizontal drift
	const wobble = 15 + Math.random() * 20;  // sinusoidal amplitude
	const size = 6 + Math.random() * 4;      // 6-10px
</script>

<div
	class="leaf-path"
	style:--start-x="{startX}vw"
	style:--drift="{drift}vw"
	style:--dur="{duration}s"
>
	<div
		class="leaf-wobble"
		style:--wobble="{wobble}px"
		style:--dur="{duration}s"
	>
		<svg
			class="leaf"
			width="{size}"
			height="{size * 1.3}"
			viewBox="0 0 10 13"
			style:--dur="{duration}s"
		>
			<path
				d="M5 0 C2 3, 0 6, 1 10 C2 12, 4 13, 5 13 C6 13, 8 12, 9 10 C10 6, 8 3, 5 0Z"
				fill="{color}"
				opacity="0.7"
			/>
			<line x1="5" y1="1" x2="5" y2="12" stroke="{color}" stroke-width="0.5" opacity="0.5" />
		</svg>
	</div>
</div>

<style>
	.leaf-path {
		position: fixed;
		left: var(--start-x);
		top: -20px;
		pointer-events: none;
		animation: leaf-fall var(--dur) ease-in forwards;
	}

	.leaf-wobble {
		animation: leaf-sway var(--dur) ease-in-out;
	}

	.leaf {
		animation: leaf-spin var(--dur) linear;
		display: block;
	}

	@keyframes leaf-fall {
		0%   { transform: translateY(0) translateX(0); opacity: 0; }
		5%   { opacity: 0.8; }
		90%  { opacity: 0.6; }
		100% { transform: translateY(110vh) translateX(var(--drift)); opacity: 0; }
	}

	@keyframes leaf-sway {
		0%   { transform: translateX(0); }
		20%  { transform: translateX(var(--wobble)); }
		40%  { transform: translateX(calc(var(--wobble) * -0.7)); }
		60%  { transform: translateX(calc(var(--wobble) * 0.5)); }
		80%  { transform: translateX(calc(var(--wobble) * -0.3)); }
		100% { transform: translateX(0); }
	}

	@keyframes leaf-spin {
		0%   { transform: rotate(0deg); }
		25%  { transform: rotate(45deg); }
		50%  { transform: rotate(-15deg); }
		75%  { transform: rotate(30deg); }
		100% { transform: rotate(5deg); }
	}
</style>
