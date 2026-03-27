<script lang="ts">
	import { tick } from 'svelte';
	import { textLog, streamingActive } from '../stores/game';
	import type { TextLogEntry } from '$lib/types';

	let logEl: HTMLDivElement;

	$effect(() => {
		// Scroll to bottom when log changes
		const _ = $textLog;
		tick().then(() => {
			if (logEl) {
				logEl.scrollTop = logEl.scrollHeight;
			}
		});
	});

	function entryClass(entry: TextLogEntry): string {
		if (entry.source === 'player') return 'entry player';
		if (entry.source === 'system') return 'entry system';
		return 'entry npc';
	}
</script>

<div class="chat-panel" bind:this={logEl}>
	{#each $textLog as entry (entry)}
		<div class={entryClass(entry)}>
			{#if entry.source !== 'system'}
				<span class="source">{entry.source === 'player' ? 'You' : entry.source}:</span>
			{/if}
			<span class="content">{entry.content}{#if entry.streaming}<span class="cursor">▋</span>{/if}</span>
		</div>
	{/each}
	{#if $streamingActive && ($textLog.length === 0 || !$textLog[$textLog.length - 1].streaming)}
		<div class="spinner-row">
			<span class="spinner" aria-label="Loading">
				<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
					<path class="strand strand-b"
						d="M 18,2 C 18,8 6,8 6,12 C 6,16 18,16 18,22"
						fill="none" stroke-width="2.5" stroke-linecap="round" />
					<path class="strand strand-a"
						d="M 6,2 C 6,8 18,8 18,12 C 18,16 6,16 6,22"
						fill="none" stroke-width="2.5" stroke-linecap="round" />
				</svg>
			</span>
		</div>
	{/if}
</div>

<style>
	.chat-panel {
		flex: 1;
		overflow-y: auto;
		padding: 1rem;
		display: flex;
		flex-direction: column;
		gap: 0.6rem;
		background: var(--color-bg);
	}

	.entry {
		line-height: 1.6;
		font-size: 1.15rem;
		white-space: pre-wrap;
	}

	.source {
		font-weight: 600;
		margin-right: 0.5rem;
	}

	.player .source {
		color: var(--color-muted);
	}

	.npc .source {
		color: var(--color-accent);
	}

	.system .content {
		color: var(--color-fg);
	}

	.cursor {
		display: inline-block;
		animation: blink 1s step-end infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0; }
	}

	.spinner-row {
		display: flex;
		align-items: center;
		padding: 0.25rem 0;
	}

	.spinner {
		display: inline-block;
		width: 1.5rem;
		height: 1.5rem;
	}

	.spinner svg {
		width: 100%;
		height: 100%;
	}

	.strand {
		stroke-dasharray: 40;
		stroke-dashoffset: 40;
		animation: knit 2.4s ease-in-out infinite;
	}

	.strand-a {
		stroke: var(--color-accent);
	}

	.strand-b {
		stroke: var(--color-muted);
		animation-delay: 0.15s;
	}

	@keyframes knit {
		0%   { stroke-dashoffset: 40; opacity: 0.3; }
		10%  { opacity: 1; }
		60%  { stroke-dashoffset: 0; opacity: 1; }
		80%  { stroke-dashoffset: 0; opacity: 0; }
		100% { stroke-dashoffset: 40; opacity: 0; }
	}

	@media (prefers-reduced-motion: reduce) {
		.strand {
			animation: pulse 2s ease-in-out infinite;
			stroke-dashoffset: 0;
		}

		@keyframes pulse {
			0%, 100% { opacity: 0.5; }
			50% { opacity: 1; }
		}
	}
</style>
