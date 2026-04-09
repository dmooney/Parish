<!--
  RainInkBleed — adds subtle text-shadow to body during heavy rain,
  as if ink is bleeding on wet paper. Controlled via a body class.
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	onMount(() => { document.body.classList.add('rain-ink-bleed'); });
	onDestroy(() => { document.body.classList.remove('rain-ink-bleed'); });
</script>
