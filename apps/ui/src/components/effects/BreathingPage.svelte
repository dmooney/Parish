<!--
  BreathingPage — the paper texture overlay slowly "breathes."
  Applied via a body class to animate the existing ::after pseudo-element.
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	onMount(() => { document.body.classList.add('breathing-page'); });
	onDestroy(() => { document.body.classList.remove('breathing-page'); });
</script>
