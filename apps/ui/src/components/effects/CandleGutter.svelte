<!--
  CandleGutter — the vignette deepens when the player is idle at night,
  as if a candle is guttering down. Applied via a body class.
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	onMount(() => { document.body.classList.add('candle-gutter'); });
	onDestroy(() => { document.body.classList.remove('candle-gutter'); });
</script>
