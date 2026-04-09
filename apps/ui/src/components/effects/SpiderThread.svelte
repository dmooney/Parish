<!--
  SpiderThread — a tiny spider lowers itself from the top of the window
  on a thin line, dangles, then climbs back up. Maximum subtlety.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	const x = 20 + Math.random() * 60;
	const dropDistance = 60 + Math.random() * 80;
</script>

<div
	class="spider-wrap"
	style:left="{x}%"
	style:--drop="{dropDistance}px"
>
	<div class="thread"></div>
	<svg class="spider" width="6" height="6" viewBox="0 0 10 10">
		<circle cx="5" cy="5" r="3" fill="var(--color-fg, #31240f)" opacity="0.4" />
		<circle cx="5" cy="3" r="1.5" fill="var(--color-fg, #31240f)" opacity="0.4" />
		<!-- legs -->
		<g stroke="var(--color-fg, #31240f)" stroke-width="0.4" opacity="0.3">
			<line x1="3" y1="4" x2="0" y2="2" />
			<line x1="3" y1="5" x2="0" y2="5" />
			<line x1="3" y1="6" x2="0" y2="8" />
			<line x1="7" y1="4" x2="10" y2="2" />
			<line x1="7" y1="5" x2="10" y2="5" />
			<line x1="7" y1="6" x2="10" y2="8" />
		</g>
	</svg>
</div>

<style>
	.spider-wrap {
		position: fixed;
		top: 0;
		pointer-events: none;
		z-index: 410;
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.thread {
		width: 0.5px;
		height: 0;
		background: rgba(100, 90, 70, 0.2);
		animation: thread-extend 4s ease-out forwards, thread-retract 3s ease-in 8s forwards;
	}

	.spider {
		animation: spider-drop 4s ease-out forwards,
		           spider-dangle 2s ease-in-out 4s 2,
		           spider-climb 3s ease-in 8s forwards;
		opacity: 0;
	}

	@keyframes thread-extend {
		0%   { height: 0; }
		100% { height: var(--drop); }
	}

	@keyframes thread-retract {
		0%   { height: var(--drop); }
		100% { height: 0; }
	}

	@keyframes spider-drop {
		0%   { opacity: 0; transform: translateY(0); }
		10%  { opacity: 0.5; }
		100% { opacity: 0.5; transform: translateY(0); }
	}

	@keyframes spider-dangle {
		0%, 100% { transform: rotate(0deg) translateY(0); }
		25%      { transform: rotate(3deg) translateY(1px); }
		75%      { transform: rotate(-3deg) translateY(-1px); }
	}

	@keyframes spider-climb {
		0%   { opacity: 0.5; }
		100% { opacity: 0; transform: translateY(calc(var(--drop) * -1)); }
	}
</style>
