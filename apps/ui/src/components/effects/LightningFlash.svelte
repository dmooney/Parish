<!--
  LightningFlash — a sudden white flash + screen-shake during storms.
  Pure CSS animation, auto-plays on mount.
-->
<script lang="ts">
	import type { ActiveEffect } from '$lib/effects';

	interface Props {
		effect: ActiveEffect;
	}
	let { effect }: Props = $props();
</script>

<div class="lightning"></div>

<style>
	.lightning {
		position: fixed;
		inset: 0;
		pointer-events: none;
		background: white;
		animation: flash 800ms ease-out forwards, shake 300ms ease-out;
	}

	@keyframes flash {
		0%   { opacity: 0; }
		15%  { opacity: 0.7; }
		40%  { opacity: 0.25; }
		100% { opacity: 0; }
	}

	@keyframes shake {
		0%   { transform: translate(0, 0); }
		20%  { transform: translate(-2px, 1px); }
		40%  { transform: translate(2px, -1px); }
		60%  { transform: translate(-1px, -2px); }
		80%  { transform: translate(1px, 2px); }
		100% { transform: translate(0, 0); }
	}
</style>
