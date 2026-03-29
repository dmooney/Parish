<script lang="ts">
	import { savePickerVisible, saveFiles, currentSaveState } from '../stores/save';
	import { discoverSaveFiles, loadBranch, saveGame, newSaveFile, createBranch, getSaveState, getWorldSnapshot, getMap, getNpcsHere } from '$lib/ipc';
	import { worldState, mapData, npcsHere } from '../stores/game';
	import type { SaveFileInfo, SaveBranchDisplay } from '$lib/types';

	let loading = false;
	let forkingFileIdx: number | null = null;
	let forkingBranchId: number | null = null;
	let forkName = '';

	async function refreshSaves() {
		loading = true;
		try {
			const files = await discoverSaveFiles();
			saveFiles.set(files);
			const state = await getSaveState();
			currentSaveState.set(state);
		} catch (e) {
			console.error('Failed to discover saves:', e);
		}
		loading = false;
	}

	async function refreshGameState() {
		try {
			const [ws, md, npcs] = await Promise.all([
				getWorldSnapshot(),
				getMap(),
				getNpcsHere()
			]);
			worldState.set(ws);
			mapData.set(md);
			npcsHere.set(npcs);
		} catch (e) {
			console.error('Failed to refresh game state:', e);
		}
	}

	async function handleLoad(file: SaveFileInfo, branch: SaveBranchDisplay) {
		loading = true;
		try {
			await loadBranch(file.path, branch.id);
			await refreshGameState();
			savePickerVisible.set(false);
		} catch (e) {
			console.error('Load failed:', e);
		}
		loading = false;
	}

	async function handleSaveHere() {
		loading = true;
		try {
			await saveGame();
			await refreshSaves();
		} catch (e) {
			console.error('Save failed:', e);
		}
		loading = false;
	}

	async function handleNewGame() {
		loading = true;
		try {
			await newSaveFile();
			await refreshGameState();
			savePickerVisible.set(false);
		} catch (e) {
			console.error('New game failed:', e);
		}
		loading = false;
	}

	async function handleFork(file: SaveFileInfo, parentBranch: SaveBranchDisplay) {
		if (!forkName.trim()) return;
		loading = true;
		try {
			await loadBranch(file.path, parentBranch.id);
			await createBranch(forkName.trim());
			forkingBranchId = null;
			forkName = '';
			await refreshSaves();
		} catch (e) {
			console.error('Fork failed:', e);
		}
		loading = false;
	}

	function startFork(fileIdx: number, branchId: number) {
		forkingFileIdx = fileIdx;
		forkingBranchId = branchId;
		forkName = '';
	}

	function cancelFork() {
		forkingFileIdx = null;
		forkingBranchId = null;
		forkName = '';
	}

	function close() {
		savePickerVisible.set(false);
		forkingFileIdx = null;
		forkingBranchId = null;
		forkName = '';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			close();
		}
	}

	function getRootBranches(branches: SaveBranchDisplay[]): SaveBranchDisplay[] {
		return branches.filter(b => b.parent_name === null);
	}

	function getChildren(branches: SaveBranchDisplay[], parentName: string): SaveBranchDisplay[] {
		return branches.filter(b => b.parent_name === parentName);
	}

	function isLastRoot(branches: SaveBranchDisplay[], branch: SaveBranchDisplay, idx: number): boolean {
		const roots = getRootBranches(branches);
		const children = getChildren(branches, branch.name);
		return idx === roots.length - 1 && children.length === 0;
	}

	function connector(last: boolean): string {
		return last ? '\u2514\u2500' : '\u251c\u2500';
	}

	function indent(last: boolean): string {
		return last ? '  ' : '\u2502 ';
	}

	// Refresh saves when the picker opens
	$: if ($savePickerVisible) {
		refreshSaves();
	}

	$: files = $saveFiles;
	$: saveState = $currentSaveState;
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $savePickerVisible}
	<div class="overlay" role="dialog" aria-modal="true" aria-label="Save Files">
		<div class="modal">
			<div class="modal-header">
				<span class="modal-title">SAVE FILES</span>
				<button class="modal-close" on:click={close}>X</button>
			</div>

			<div class="modal-body">
				{#if loading && files.length === 0}
					<div class="loading-msg">Scanning save files...</div>
				{/if}

				{#each files as file, fileIdx}
					<div class="save-file">
						<div class="file-header">
							<span class="file-number">{fileIdx + 1}.</span>
							<span class="file-name">{file.filename}</span>
						</div>

						{#each getRootBranches(file.branches) as branch, branchIdx}
							{@const lastRoot = isLastRoot(file.branches, branch, branchIdx)}
							{@const children = getChildren(file.branches, branch.name)}
							<div class="branch-row">
								<span class="tree-connector">{connector(lastRoot)}</span>
								<span class="branch-name">{branch.name}</span>
								<span class="branch-meta">
									{branch.latest_location ?? 'New'}
									{#if branch.latest_game_date}
										, {branch.latest_game_date}
									{/if}
									({branch.snapshot_count} {branch.snapshot_count === 1 ? 'save' : 'saves'})
								</span>
								<span class="branch-actions">
									<button class="action-btn" on:click={() => handleLoad(file, branch)} disabled={loading}>Load</button>
									<button class="action-btn" on:click={() => handleSaveHere()} disabled={loading}>Save</button>
									<button class="action-btn" on:click={() => startFork(fileIdx, branch.id)} disabled={loading}>Fork</button>
								</span>
							</div>

							{#if forkingFileIdx === fileIdx && forkingBranchId === branch.id}
								<div class="fork-input-row">
									<span class="tree-indent">{indent(lastRoot)}</span>
									<input
										class="fork-input"
										type="text"
										placeholder="Branch name..."
										bind:value={forkName}
										on:keydown={(e) => { if (e.key === 'Enter') handleFork(file, branch); if (e.key === 'Escape') cancelFork(); }}
									/>
									<button class="action-btn" on:click={() => handleFork(file, branch)} disabled={loading || !forkName.trim()}>Create</button>
									<button class="action-btn" on:click={cancelFork}>Cancel</button>
								</div>
							{/if}

							{#each children as child, childIdx}
								{@const lastChild = childIdx === children.length - 1}
								<div class="branch-row child">
									<span class="tree-indent">{indent(lastRoot)}</span>
									<span class="tree-connector">{connector(lastChild)}</span>
									<span class="branch-name">{child.name}</span>
									<span class="branch-meta">
										{child.latest_location ?? 'New'}
										{#if child.latest_game_date}
											, {child.latest_game_date}
										{/if}
										({child.snapshot_count} {child.snapshot_count === 1 ? 'save' : 'saves'})
									</span>
									<span class="branch-actions">
										<button class="action-btn" on:click={() => handleLoad(file, child)} disabled={loading}>Load</button>
										<button class="action-btn" on:click={() => handleSaveHere()} disabled={loading}>Save</button>
										<button class="action-btn" on:click={() => startFork(fileIdx, child.id)} disabled={loading}>Fork</button>
									</span>
								</div>

								{#if forkingFileIdx === fileIdx && forkingBranchId === child.id}
									<div class="fork-input-row">
										<span class="tree-indent">{indent(lastRoot)}</span>
										<span class="tree-indent">{indent(lastChild)}</span>
										<input
											class="fork-input"
											type="text"
											placeholder="Branch name..."
											bind:value={forkName}
											on:keydown={(e) => { if (e.key === 'Enter') handleFork(file, child); if (e.key === 'Escape') cancelFork(); }}
										/>
										<button class="action-btn" on:click={() => handleFork(file, child)} disabled={loading || !forkName.trim()}>Create</button>
										<button class="action-btn" on:click={cancelFork}>Cancel</button>
									</div>
								{/if}
							{/each}
						{/each}
					</div>
				{/each}

				<div class="save-file new-game">
					<button class="new-game-btn" on:click={handleNewGame} disabled={loading}>
						N. New Game
					</button>
				</div>

				{#if saveState?.filename}
					<div class="current-save">
						Current: {saveState.filename} (branch: {saveState.branch_name ?? 'main'})
						<button class="action-btn" on:click={handleSaveHere} disabled={loading}>Quick Save</button>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--color-panel-bg);
		border: 1px solid var(--color-border);
		max-width: 650px;
		width: 90%;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		border-radius: 2px;
	}

	.modal-header {
		padding: 0.6rem 0.75rem;
		border-bottom: 1px solid var(--color-border);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.modal-title {
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		color: var(--color-accent);
	}

	.modal-close {
		background: none;
		border: none;
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.75rem;
		padding: 0.2rem 0.4rem;
	}
	.modal-close:hover {
		color: var(--color-fg);
	}

	.modal-body {
		flex: 1;
		overflow-y: auto;
		padding: 0.75rem;
	}

	.save-file {
		margin-bottom: 0.75rem;
		border-bottom: 1px solid var(--color-border);
		padding-bottom: 0.5rem;
	}
	.save-file:last-child {
		border-bottom: none;
	}

	.file-header {
		display: flex;
		align-items: baseline;
		gap: 0.4rem;
		margin-bottom: 0.3rem;
	}

	.file-number {
		color: var(--color-muted);
		font-size: 0.8rem;
	}

	.file-name {
		color: var(--color-accent);
		font-size: 0.85rem;
	}

	.branch-row {
		display: flex;
		align-items: baseline;
		gap: 0.3rem;
		padding: 0.2rem 0;
		padding-left: 1.2rem;
		font-size: 0.8rem;
	}
	.branch-row.child {
		padding-left: 1.2rem;
	}
	.branch-row:hover {
		background: var(--color-input-bg);
	}

	.tree-connector {
		color: var(--color-muted);
		font-family: monospace;
		flex-shrink: 0;
	}

	.tree-indent {
		color: var(--color-muted);
		font-family: monospace;
		flex-shrink: 0;
	}

	.branch-name {
		color: var(--color-accent);
		font-weight: bold;
		flex-shrink: 0;
	}

	.branch-meta {
		color: var(--color-muted);
		font-size: 0.75rem;
		flex: 1;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.branch-actions {
		display: flex;
		gap: 0.3rem;
		flex-shrink: 0;
	}

	.action-btn {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.65rem;
		padding: 0.1rem 0.4rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.action-btn:hover:not(:disabled) {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}
	.action-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.fork-input-row {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		padding: 0.2rem 0;
		padding-left: 1.2rem;
		font-size: 0.8rem;
	}

	.fork-input {
		background: var(--color-input-bg);
		border: 1px solid var(--color-border);
		color: var(--color-fg);
		font-size: 0.75rem;
		padding: 0.15rem 0.4rem;
		flex: 1;
		max-width: 200px;
	}
	.fork-input:focus {
		border-color: var(--color-accent);
		outline: none;
	}

	.new-game {
		border-bottom: none;
	}

	.new-game-btn {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-fg);
		cursor: pointer;
		font-size: 0.85rem;
		padding: 0.4rem 0.75rem;
		width: 100%;
		text-align: left;
	}
	.new-game-btn:hover:not(:disabled) {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}
	.new-game-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.current-save {
		font-size: 0.7rem;
		color: var(--color-muted);
		padding-top: 0.5rem;
		border-top: 1px solid var(--color-border);
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.loading-msg {
		color: var(--color-muted);
		font-size: 0.8rem;
		font-style: italic;
		padding: 1rem 0;
		text-align: center;
	}
</style>
