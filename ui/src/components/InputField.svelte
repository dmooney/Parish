<script lang="ts">
	import { streamingActive, npcsHere } from '../stores/game';
	import { submitInput } from '$lib/ipc';

	let inputEl: HTMLInputElement;
	let text = $state('');
	let showMentions = $state(false);
	let selectedIndex = $state(0);
	let mentionQuery = $state('');

	const filteredNpcs = $derived(
		$npcsHere.filter((npc) =>
			npc.name.toLowerCase().startsWith(mentionQuery.toLowerCase())
		)
	);

	$effect(() => {
		if (!$streamingActive && inputEl) {
			inputEl.focus();
		}
	});

	$effect(() => {
		// Reset selection when filtered list changes
		if (selectedIndex >= filteredNpcs.length) {
			selectedIndex = Math.max(0, filteredNpcs.length - 1);
		}
	});

	function detectMention() {
		if (!inputEl) return;
		const value = text;
		// Only trigger when @ is at the start of input
		if (value.startsWith('@')) {
			const afterAt = value.slice(1);
			// Extract the mention query (everything up to end or first lowercase-started word after a name)
			const spaceIdx = afterAt.indexOf(' ');
			// Show dropdown while typing the name portion
			if (spaceIdx === -1) {
				mentionQuery = afterAt;
				showMentions = afterAt.length > 0 && $npcsHere.length > 0;
			} else {
				// Check if there's still a plausible name being typed
				// e.g., "@Padraig D" should still show dropdown
				const words = afterAt.split(' ');
				const allCapitalized = words.every(
					(w) => w.length === 0 || w[0] === w[0].toUpperCase()
				);
				if (allCapitalized && words[words.length - 1] === '') {
					// Trailing space after capitalized words — could be continuing a name
					mentionQuery = afterAt.trimEnd();
					showMentions = $npcsHere.length > 0;
				} else if (allCapitalized) {
					mentionQuery = afterAt;
					showMentions = $npcsHere.length > 0;
				} else {
					showMentions = false;
				}
			}
		} else {
			showMentions = false;
		}
		selectedIndex = 0;
	}

	function selectNpc(npcName: string) {
		// Use first name only for brevity
		const firstName = npcName.split(' ')[0];
		const afterMention = text.startsWith('@') ? getTextAfterMention() : '';
		text = `@${firstName} ${afterMention}`;
		showMentions = false;
		inputEl?.focus();
		// Move cursor to end
		requestAnimationFrame(() => {
			if (inputEl) {
				inputEl.selectionStart = inputEl.selectionEnd = text.length;
			}
		});
	}

	function getTextAfterMention(): string {
		const value = text;
		if (!value.startsWith('@')) return value;
		const afterAt = value.slice(1);
		const words = afterAt.split(' ');
		// Skip capitalized words (part of the mention)
		let i = 0;
		for (; i < words.length; i++) {
			if (words[i].length > 0 && words[i][0] !== words[i][0].toUpperCase()) {
				break;
			}
		}
		return words.slice(i).join(' ');
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (showMentions && filteredNpcs.length > 0) {
			selectNpc(filteredNpcs[selectedIndex].name);
			return;
		}
		const trimmed = text.trim();
		if (!trimmed || $streamingActive) return;
		text = '';
		showMentions = false;
		await submitInput(trimmed);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (showMentions && filteredNpcs.length > 0) {
			if (e.key === 'ArrowDown') {
				e.preventDefault();
				selectedIndex = (selectedIndex + 1) % filteredNpcs.length;
				return;
			}
			if (e.key === 'ArrowUp') {
				e.preventDefault();
				selectedIndex =
					(selectedIndex - 1 + filteredNpcs.length) % filteredNpcs.length;
				return;
			}
			if (e.key === 'Tab') {
				e.preventDefault();
				selectNpc(filteredNpcs[selectedIndex].name);
				return;
			}
			if (e.key === 'Escape') {
				e.preventDefault();
				showMentions = false;
				return;
			}
		}
		if (e.key === 'Enter') {
			handleSubmit(e);
		}
	}

	function handleInput() {
		detectMention();
	}
</script>

<div class="input-wrapper">
	{#if showMentions && filteredNpcs.length > 0}
		<ul class="mention-dropdown" role="listbox" aria-label="Mention NPC">
			{#each filteredNpcs as npc, i}
				<li
					role="option"
					aria-selected={i === selectedIndex}
					class="mention-item"
					class:selected={i === selectedIndex}
					onmousedown={(e) => { e.preventDefault(); selectNpc(npc.name); }}
					onmouseenter={() => (selectedIndex = i)}
				>
					<span class="mention-name">{npc.name}</span>
					{#if npc.introduced}
						<span class="mention-detail">{npc.occupation}</span>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
	<form class="input-form" onsubmit={handleSubmit}>
		<input
			bind:this={inputEl}
			bind:value={text}
			onkeydown={handleKeydown}
			oninput={handleInput}
			disabled={$streamingActive}
			placeholder={$streamingActive ? 'Waiting…' : 'What do you do? (@ to mention NPC)'}
			class="input-field"
			autocomplete="off"
			spellcheck="false"
		/>
		<button type="submit" disabled={$streamingActive || !text.trim()} class="send-btn">
			Send
		</button>
	</form>
</div>

<style>
	.input-wrapper {
		position: relative;
	}

	.input-form {
		display: flex;
		gap: 0.5rem;
		padding: 0.6rem 0.75rem;
		background: var(--color-panel-bg);
		border-top: 1px solid var(--color-border);
	}

	.input-field {
		flex: 1;
		background: var(--color-input-bg);
		border: 1px solid var(--color-border);
		color: var(--color-fg);
		padding: 0.5rem 0.75rem;
		font-size: 0.95rem;
		font-family: inherit;
		border-radius: 4px;
		outline: none;
	}

	.input-field:focus {
		border-color: var(--color-accent);
	}

	.input-field:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.input-field::placeholder {
		color: var(--color-muted);
	}

	.send-btn {
		background: var(--color-accent);
		color: var(--color-bg);
		border: none;
		padding: 0.5rem 1rem;
		font-size: 0.85rem;
		font-family: inherit;
		font-weight: 600;
		border-radius: 4px;
		cursor: pointer;
		transition: opacity 0.15s;
	}

	.send-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.send-btn:hover:not(:disabled) {
		opacity: 0.85;
	}

	.mention-dropdown {
		position: absolute;
		bottom: 100%;
		left: 0.75rem;
		right: 0.75rem;
		margin: 0;
		padding: 0.25rem 0;
		list-style: none;
		background: var(--color-panel-bg);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		box-shadow: 0 -2px 8px rgba(0, 0, 0, 0.3);
		max-height: 12rem;
		overflow-y: auto;
		z-index: 10;
	}

	.mention-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.4rem 0.75rem;
		cursor: pointer;
		color: var(--color-fg);
		font-size: 0.9rem;
	}

	.mention-item.selected {
		background: var(--color-accent);
		color: var(--color-bg);
	}

	.mention-name {
		font-weight: 600;
	}

	.mention-detail {
		font-size: 0.8rem;
		opacity: 0.7;
	}

	.mention-item.selected .mention-detail {
		opacity: 0.85;
	}
</style>
