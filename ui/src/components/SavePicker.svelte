<script lang="ts">
	import { savePickerVisible, saveFiles, currentSaveState } from '../stores/save';
	import { discoverSaveFiles, loadBranch, saveGame, newSaveFile, createBranch, getSaveState, getWorldSnapshot, getMap, getNpcsHere } from '$lib/ipc';
	import { worldState, mapData, npcsHere } from '../stores/game';
	import type { SaveFileInfo, SaveBranchDisplay } from '$lib/types';

	let loading = false;
	let forkingBranchId: number | null = null;
	let forkName = '';
	/** When true, show the multi-file "ledgers" view instead of branches. */
	let showLedgers = false;

	/** The active save file (matched by filename from save state). */
	$: activeFile = files.find(f => f.filename === saveState?.filename) ?? files[0] ?? null;

	async function refreshSaves() {
		loading = true;
		try {
			const allFiles = await discoverSaveFiles();
			saveFiles.set(allFiles);
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
			showLedgers = false;
			savePickerVisible.set(false);
		} catch (e) {
			console.error('New game failed:', e);
		}
		loading = false;
	}

	async function handleSwitchLedger(file: SaveFileInfo) {
		// Load the first branch of the selected file
		const branch = file.branches[0];
		if (!branch) return;
		loading = true;
		try {
			await loadBranch(file.path, branch.id);
			await refreshGameState();
			showLedgers = false;
			await refreshSaves();
		} catch (e) {
			console.error('Switch ledger failed:', e);
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

	function startFork(branchId: number) {
		forkingBranchId = branchId;
		forkName = '';
	}

	function cancelFork() {
		forkingBranchId = null;
		forkName = '';
	}

	function close() {
		savePickerVisible.set(false);
		forkingBranchId = null;
		forkName = '';
		showLedgers = false;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (showLedgers) {
				showLedgers = false;
			} else {
				close();
			}
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
	<div class="overlay" role="dialog" aria-modal="true" aria-label="The Parish Ledger">
		<div class="modal">
			<div class="modal-header">
				<span class="modal-title">
					{#if showLedgers}
						Ledgers
					{:else}
						The Parish Ledger
					{/if}
				</span>
				<button class="modal-close" on:click={close}>X</button>
			</div>

			<div class="modal-body">
				{#if loading && files.length === 0}
					<div class="loading-msg">Scanning save files...</div>
				{/if}

				{#if showLedgers}
					<!-- Multi-file ledger picker -->
					{#each files as file, fileIdx}
						{@const isActive = file.filename === saveState?.filename}
						<div class="ledger-row" class:ledger-active={isActive}>
							<span class="file-number">{fileIdx + 1}.</span>
							<span class="file-name">{file.filename}</span>
							<span class="branch-meta">
								{file.branches.length} {file.branches.length === 1 ? 'branch' : 'branches'}
								{#if file.branches[0]?.latest_location}
									— {file.branches[0].latest_location}
								{/if}
							</span>
							{#if isActive}
								<span class="ledger-current">current</span>
							{:else}
								<button class="action-btn" on:click={() => handleSwitchLedger(file)} disabled={loading}>Open</button>
							{/if}
						</div>
					{/each}

					<div class="ledger-row new-ledger" on:click={handleNewGame} role="button" tabindex="0" on:keydown={(e) => { if (e.key === 'Enter') handleNewGame(); }}>
						<span class="file-number">+</span>
						<span class="file-name">Fork New Ledger</span>
					</div>
				{:else}
					<!-- Branch view for the active save file -->
					{#if activeFile}
						{#each getRootBranches(activeFile.branches) as branch, branchIdx}
							{@const lastRoot = isLastRoot(activeFile.branches, branch, branchIdx)}
							{@const children = getChildren(activeFile.branches, branch.name)}
							{@const isCurrent = branch.name === saveState?.branch_name}
							<div class="branch-row" class:branch-current={isCurrent}>
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
									{#if isCurrent}
										<span class="current-marker">current</span>
									{:else}
										<button class="action-btn" on:click={() => handleLoad(activeFile, branch)} disabled={loading}>Load</button>
									{/if}
									<button class="action-btn" on:click={() => startFork(branch.id)} disabled={loading}>New Branch</button>
								</span>
							</div>

							{#if forkingBranchId === branch.id}
								<div class="fork-input-row">
									<span class="tree-indent">{indent(lastRoot)}</span>
									<input
										class="fork-input"
										type="text"
										placeholder="New branch name..."
										bind:value={forkName}
										on:keydown={(e) => { if (e.key === 'Enter') handleFork(activeFile, branch); if (e.key === 'Escape') cancelFork(); }}
									/>
									<button class="action-btn" on:click={() => handleFork(activeFile, branch)} disabled={loading || !forkName.trim()}>Create</button>
									<button class="action-btn" on:click={cancelFork}>Cancel</button>
								</div>
							{/if}

							{#each children as child, childIdx}
								{@const lastChild = childIdx === children.length - 1}
								{@const isChildCurrent = child.name === saveState?.branch_name}
								<div class="branch-row child" class:branch-current={isChildCurrent}>
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
										{#if isChildCurrent}
											<span class="current-marker">current</span>
										{:else}
											<button class="action-btn" on:click={() => handleLoad(activeFile, child)} disabled={loading}>Load</button>
										{/if}
										<button class="action-btn" on:click={() => startFork(child.id)} disabled={loading}>New Branch</button>
									</span>
								</div>

								{#if forkingBranchId === child.id}
									<div class="fork-input-row">
										<span class="tree-indent">{indent(lastRoot)}</span>
										<span class="tree-indent">{indent(lastChild)}</span>
										<input
											class="fork-input"
											type="text"
											placeholder="New branch name..."
											bind:value={forkName}
											on:keydown={(e) => { if (e.key === 'Enter') handleFork(activeFile, child); if (e.key === 'Escape') cancelFork(); }}
										/>
										<button class="action-btn" on:click={() => handleFork(activeFile, child)} disabled={loading || !forkName.trim()}>Create</button>
										<button class="action-btn" on:click={cancelFork}>Cancel</button>
									</div>
								{/if}
							{/each}
						{/each}
					{:else}
						<div class="loading-msg">No save file found.</div>
					{/if}
				{/if}
			</div>

			<div class="modal-footer">
				{#if showLedgers}
					<button class="footer-btn" on:click={() => { showLedgers = false; }}>
						← Back
					</button>
				{:else}
					<button class="footer-btn" on:click={() => { showLedgers = true; }}>
						Ledgers
					</button>
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
		height: 67vh;
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

	.modal-footer {
		padding: 0.4rem 0.75rem;
		border-top: 1px solid var(--color-border);
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.footer-btn {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.65rem;
		padding: 0.15rem 0.5rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}
	.footer-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	/* ── Branch view ──────────────────────────────────────────────── */

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
	.branch-row.branch-current {
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
		align-items: center;
	}

	.current-marker {
		font-size: 0.6rem;
		color: var(--color-muted);
		font-style: italic;
		text-transform: uppercase;
		letter-spacing: 0.05em;
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

	/* ── Ledger view ─────────────────────────────────────────────── */

	.ledger-row {
		display: flex;
		align-items: baseline;
		gap: 0.4rem;
		padding: 0.35rem 0.5rem;
		font-size: 0.8rem;
		border-bottom: 1px solid var(--color-border);
	}
	.ledger-row:last-child {
		border-bottom: none;
	}
	.ledger-row:hover {
		background: var(--color-input-bg);
	}
	.ledger-row.ledger-active {
		background: var(--color-input-bg);
	}

	.file-number {
		color: var(--color-muted);
		font-size: 0.8rem;
		flex-shrink: 0;
	}

	.file-name {
		color: var(--color-accent);
		font-size: 0.85rem;
		flex-shrink: 0;
	}

	.ledger-current {
		font-size: 0.6rem;
		color: var(--color-muted);
		font-style: italic;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.new-ledger {
		border-bottom: none;
		cursor: pointer;
	}

	.loading-msg {
		color: var(--color-muted);
		font-size: 0.8rem;
		font-style: italic;
		padding: 1rem 0;
		text-align: center;
	}
</style>
