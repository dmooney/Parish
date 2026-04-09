<!--
  ImbolcThaw — tiny white snowdrop-like dots at the bottom of the window
  with a warming light. Spring stirring beneath winter.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	let visible = $state(false);
	onMount(() => { requestAnimationFrame(() => { visible = true; }); });

	const drops = Array.from({ length: 4 + Math.floor(Math.random() * 4) }, () => ({
		x: 10 + Math.random() * 80,
		delay: Math.random() * 5,
		size: 2 + Math.random() * 2,
	}));
</script>

<div class="imbolc" class:visible>
	<div class="warmth"></div>
	{#each drops as d}
		<div
			class="snowdrop"
			style:left="{d.x}%"
			style:--delay="{d.delay}s"
			style:--size="{d.size}px"
		></div>
	{/each}
</div>

<style>
	.imbolc {
		position: fixed;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 4s ease-in;
	}
	.imbolc.visible { opacity: 1; }

	.warmth {
		position: absolute;
		inset: 0;
		background: linear-gradient(to top, rgba(255, 240, 200, 0.04) 0%, transparent 20%);
	}

	.snowdrop {
		position: absolute;
		bottom: 10px;
		width: var(--size);
		height: var(--size);
		border-radius: 50%;
		background: rgba(255, 255, 255, 0.6);
		box-shadow: 0 0 3px rgba(255, 255, 255, 0.3);
		animation: snowdrop-appear 6s ease-in-out var(--delay) infinite;
		opacity: 0;
	}

	@keyframes snowdrop-appear {
		0%, 100% { opacity: 0; transform: translateY(5px); }
		30%      { opacity: 0.7; transform: translateY(0); }
		70%      { opacity: 0.5; transform: translateY(-2px); }
	}
</style>
