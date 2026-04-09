<!--
  InkSplash — a ring of ink radiating from the input field on submit.
  Applied via a body class. The feeling of pressing quill to paper.
-->
<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { ActiveEffect } from '$lib/effects';

	interface Props { effect: ActiveEffect; }
	let { effect }: Props = $props();

	onMount(() => { document.body.classList.add('ink-splash'); });
	onDestroy(() => { document.body.classList.remove('ink-splash'); });
</script>
