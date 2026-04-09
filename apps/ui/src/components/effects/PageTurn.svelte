<!--
  PageTurn — brief perspective tilt when traveling to a new location.
  As if turning a page in a book.
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	onMount(() => { document.body.classList.add('page-turn'); });
	onDestroy(() => { document.body.classList.remove('page-turn'); });
</script>
