<script lang="ts">
	import { savePickerVisible, saveFiles, currentSaveState } from '../stores/save';
	import { discoverSaveFiles, loadBranch, saveGame, newSaveFile, newGame, createBranch, getSaveState, getWorldSnapshot, getMap, getNpcsHere } from '$lib/ipc';
	import { worldState, mapData, npcsHere } from '../stores/game';
	import type { SaveFileInfo, SaveBranchDisplay } from '$lib/types';

	let loading = false;
	let forkingBranchId: number | null = null;
	let forkName = '';
	let showLedgers = false;

	$: activeFile = files.find(f => f.filename === saveState?.filename) ?? files[0] ?? null;

	// ── DAG layout ──────────────────────────────────────────────────

	const NODE_W = 160;
	const NODE_H = 70;
	const GAP_X = 24;
	const GAP_Y = 44;

	interface TreeNode {
		branch: SaveBranchDisplay;
		children: TreeNode[];
		x: number;
		y: number;
		span: number; // number of leaf-slots this subtree occupies
	}

	interface LayoutResult {
		nodes: TreeNode[];
		width: number;
		height: number;
		edges: { x1: number; y1: number; x2: number; y2: number }[];
	}

	function layoutTree(branches: SaveBranchDisplay[]): LayoutResult {
		if (branches.length === 0) return { nodes: [], width: 0, height: 0, edges: [] };

		// Build tree structure
		function buildNode(branch: SaveBranchDisplay): TreeNode {
			const children = branches
				.filter(b => b.parent_name === branch.name)
				.map(b => buildNode(b));
			return { branch, children, x: 0, y: 0, span: 0 };
		}

		const roots = branches
			.filter(b => b.parent_name === null)
			.map(b => buildNode(b));

		// If somehow no roots, treat all as roots
		const tree = roots.length > 0 ? roots : branches.map(b => buildNode(b));

		// Compute spans (how many leaf slots each subtree needs)
		function computeSpan(node: TreeNode): number {
			if (node.children.length === 0) {
				node.span = 1;
			} else {
				node.span = node.children.reduce((sum, c) => sum + computeSpan(c), 0);
			}
			return node.span;
		}

		let totalSpan = 0;
		for (const root of tree) {
			totalSpan += computeSpan(root);
		}

		// Compute depth (for y positioning)
		let maxDepth = 0;
		function computeDepth(node: TreeNode, depth: number) {
			if (depth > maxDepth) maxDepth = depth;
			for (const child of node.children) {
				computeDepth(child, depth + 1);
			}
		}
		for (const root of tree) {
			computeDepth(root, 0);
		}

		// Assign x positions: each leaf gets a slot, parents center over children
		let leafSlot = 0;
		function assignX(node: TreeNode) {
			if (node.children.length === 0) {
				node.x = leafSlot * (NODE_W + GAP_X);
				leafSlot++;
			} else {
				for (const child of node.children) {
					assignX(child);
				}
				// Center parent over children
				const firstChild = node.children[0];
				const lastChild = node.children[node.children.length - 1];
				node.x = (firstChild.x + lastChild.x) / 2;
			}
		}

		for (const root of tree) {
			assignX(root);
		}

		// Assign y positions: root at bottom (maxDepth), leaves at top (0)
		// Inverted: depth 0 is at the bottom of the container
		function assignY(node: TreeNode, depth: number) {
			node.y = (maxDepth - depth) * (NODE_H + GAP_Y);
			for (const child of node.children) {
				assignY(child, depth + 1);
			}
		}
		for (const root of tree) {
			assignY(root, 0);
		}

		// Collect all nodes flat
		const allNodes: TreeNode[] = [];
		function collectNodes(node: TreeNode) {
			allNodes.push(node);
			for (const child of node.children) {
				collectNodes(child);
			}
		}
		for (const root of tree) {
			collectNodes(root);
		}

		// Compute edges (parent bottom-center → child top-center)
		const edges: { x1: number; y1: number; x2: number; y2: number }[] = [];
		function collectEdges(node: TreeNode) {
			for (const child of node.children) {
				edges.push({
					x1: node.x + NODE_W / 2,
					y1: node.y,       // top of parent (parent is below)
					x2: child.x + NODE_W / 2,
					y2: child.y + NODE_H // bottom of child (child is above)
				});
				collectEdges(child);
			}
		}
		for (const root of tree) {
			collectEdges(root);
		}

		// Compute container size
		let maxX = 0;
		for (const n of allNodes) {
			if (n.x + NODE_W > maxX) maxX = n.x + NODE_W;
		}
		const width = maxX;
		const height = (maxDepth + 1) * (NODE_H + GAP_Y) - GAP_Y;

		return { nodes: allNodes, width, height, edges };
	}

	// ── Handlers ────────────────────────────────────────────────────

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

	async function handleLoadBranch(file: SaveFileInfo, branch: SaveBranchDisplay) {
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

	async function handleForkLedger() {
		loading = true;
		try {
			await newSaveFile();
			await refreshGameState();
			showLedgers = false;
			savePickerVisible.set(false);
		} catch (e) {
			console.error('Fork ledger failed:', e);
		}
		loading = false;
	}

	async function handleNewGame() {
		loading = true;
		try {
			await newGame();
			await refreshGameState();
			showLedgers = false;
			savePickerVisible.set(false);
		} catch (e) {
			console.error('New game failed:', e);
		}
		loading = false;
	}

	async function handleSwitchLedger(file: SaveFileInfo) {
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

	async function handleFork(parentBranch: SaveBranchDisplay) {
		const name = forkName.trim();
		if (!name) return;
		loading = true;
		try {
			await createBranch(name, parentBranch.id);
			forkingBranchId = null;
			forkName = '';
			await refreshSaves();
		} catch (e: any) {
			console.error('Branch creation failed:', e);
			forkName = String(e).substring(0, 60);
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
			if (forkingBranchId !== null) {
				cancelFork();
			} else if (showLedgers) {
				showLedgers = false;
			} else {
				close();
			}
		}
	}

	let prevVisible = false;
	$: {
		const visible = $savePickerVisible;
		if (visible && !prevVisible) {
			refreshSaves();
		}
		prevVisible = visible;
	}

	$: files = $saveFiles;
	$: saveState = $currentSaveState;
	$: layout = activeFile ? layoutTree(activeFile.branches) : null;
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
			</div>

			<div class="modal-body">
				{#if loading && files.length === 0}
					<div class="loading-msg">Scanning save files...</div>
				{/if}

				{#if showLedgers}
					{#each files as file, fileIdx}
						{@const isActive = file.filename === saveState?.filename}
						<div class="ledger-row" class:ledger-active={isActive}>
							<span class="file-number">{fileIdx + 1}.</span>
							<span class="file-name">{file.filename}</span>
							<span class="ledger-meta">
								{file.file_size}
								{#if file.branches[0]?.latest_location}
									— {file.branches[0].latest_location}
								{/if}
							</span>
							{#if isActive}
								<span class="ledger-current">You are here</span>
							{:else}
								<button class="action-btn" on:click={() => handleSwitchLedger(file)} disabled={loading}>Open</button>
							{/if}
						</div>
					{/each}

					<div class="ledger-row new-ledger" on:click={handleForkLedger} role="button" tabindex="0" on:keydown={(e) => { if (e.key === 'Enter') handleForkLedger(); }}>
						<span class="file-number">+</span>
						<span class="file-name">Fork New Ledger</span>
					</div>

					<div class="ledger-row new-ledger" on:click={handleNewGame} role="button" tabindex="0" on:keydown={(e) => { if (e.key === 'Enter') handleNewGame(); }}>
						<span class="file-number">+</span>
						<span class="file-name">New Game</span>
					</div>
				{:else if layout && activeFile}
					<!-- Inverted DAG tree -->
					<div class="dag-scroll">
						<div class="dag-container" style="width: {layout.width}px; height: {layout.height}px;">
							<!-- Connection lines -->
							<svg class="dag-edges" width={layout.width} height={layout.height}>
								{#each layout.edges as edge}
									<path
										d="M {edge.x1} {edge.y1} C {edge.x1} {edge.y1 - GAP_Y * 0.5}, {edge.x2} {edge.y2 + GAP_Y * 0.5}, {edge.x2} {edge.y2}"
										fill="none"
										stroke="var(--color-border)"
										stroke-width="1.5"
									/>
								{/each}
							</svg>

							<!-- Node boxes -->
							{#each layout.nodes as node (node.branch.id)}
								{@const isCurrent = node.branch.name === saveState?.branch_name}
								<div
									class="dag-node"
									class:dag-current={isCurrent}
									style="left: {node.x}px; top: {node.y}px; width: {NODE_W}px; height: {NODE_H}px;"
								>
									<button
										class="node-body"
										disabled={loading}
										on:click={() => handleLoadBranch(activeFile, node.branch)}
									>
										<span class="node-name">{node.branch.name}</span>
										<span class="node-location">{node.branch.latest_location ?? 'New'}</span>
										<span class="node-date">{node.branch.latest_game_date ?? ''}</span>
									</button>
									{#if isCurrent}
										<span class="node-current-badge">You are here</span>
									{/if}
									<button
										class="node-branch-btn"
										disabled={loading}
										on:click|stopPropagation={() => startFork(node.branch.id)}
									>Branch From Here</button>
								</div>

								<!-- Fork input (appears above the node) -->
								{#if forkingBranchId === node.branch.id}
									<div class="dag-fork-input" style="left: {node.x}px; top: {node.y - 32}px; width: {NODE_W}px;">
										<input
											class="fork-input"
											type="text"
											placeholder="Branch name..."
											bind:value={forkName}
											on:keydown|stopPropagation={(e) => { if (e.key === 'Enter') { e.preventDefault(); handleFork(node.branch); } if (e.key === 'Escape') cancelFork(); }}
										/>
										<button class="fork-go" on:click|stopPropagation={() => handleFork(node.branch)} disabled={loading || !forkName.trim()}>OK</button>
									</div>
								{/if}
							{/each}
						</div>
					</div>
				{:else}
					<div class="loading-msg">No save file found.</div>
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
				<span class="footer-spacer"></span>
				<button class="footer-btn" on:click={close}>Close</button>
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
		max-width: 85vw;
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

	.modal-body {
		flex: 1;
		overflow: auto;
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
	.footer-spacer {
		flex: 1;
	}
	.footer-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}

	/* ── DAG tree ────────────────────────────────────────────────── */

	.dag-scroll {
		overflow: auto;
		display: flex;
		justify-content: center;
		min-height: 100%;
		align-items: flex-end;
		padding: 1rem 0;
	}

	.dag-container {
		position: relative;
		flex-shrink: 0;
	}

	.dag-edges {
		position: absolute;
		top: 0;
		left: 0;
		pointer-events: none;
	}

	.dag-node {
		position: absolute;
		border: 1px solid var(--color-border);
		background: var(--color-panel-bg);
		box-sizing: border-box;
		padding-top: 0;
	}
	.dag-node::before {
		content: '';
		position: absolute;
		top: -24px;
		left: 0;
		right: 0;
		height: 24px;
	}
	.dag-node:hover {
		border-color: var(--color-accent);
	}
	.dag-node.dag-current {
		border-color: var(--color-accent);
		border-width: 2px;
	}

	.node-body {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.15rem;
		padding: 0.3rem 0.5rem;
		width: 100%;
		height: 100%;
		background: none;
		border: none;
		color: var(--color-fg);
		cursor: pointer;
		text-align: center;
		box-sizing: border-box;
	}
	.node-body:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.node-branch-btn {
		display: none;
		position: absolute;
		bottom: 100%;
		left: 50%;
		transform: translateX(-50%);
		background: var(--color-panel-bg);
		backdrop-filter: blur(4px);
		border: 1px solid var(--color-border);
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.6rem;
		padding: 0.15rem 0.4rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		white-space: nowrap;
		margin-bottom: 4px;
		z-index: 5;
	}
	.dag-node:hover .node-branch-btn {
		display: block;
	}
	.node-branch-btn:hover {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}
	.node-branch-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.node-name {
		font-size: 0.75rem;
		font-weight: bold;
		color: var(--color-accent);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
	}

	.node-location {
		font-size: 0.6rem;
		color: var(--color-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
	}

	.node-date {
		font-size: 0.55rem;
		color: var(--color-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 100%;
	}

	.node-current-badge {
		position: absolute;
		top: -0.5rem;
		right: 0.3rem;
		font-size: 0.65rem;
		color: var(--color-accent);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		font-weight: bold;
		background: var(--color-panel-bg);
		padding: 0 0.25rem;
	}

	.dag-fork-input {
		position: absolute;
		display: flex;
		gap: 2px;
		z-index: 10;
	}

	.fork-input {
		background: var(--color-input-bg);
		border: 1px solid var(--color-border);
		color: var(--color-fg);
		font-size: 0.65rem;
		padding: 0.15rem 0.3rem;
		flex: 1;
		min-width: 0;
	}
	.fork-input:focus {
		border-color: var(--color-accent);
		outline: none;
	}

	.fork-go {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.55rem;
		padding: 0.1rem 0.3rem;
		text-transform: uppercase;
	}
	.fork-go:hover:not(:disabled) {
		color: var(--color-accent);
		border-color: var(--color-accent);
	}
	.fork-go:disabled {
		opacity: 0.4;
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

	.ledger-meta {
		color: var(--color-muted);
		font-size: 0.75rem;
		flex: 1;
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

	.action-btn {
		background: none;
		border: 1px solid var(--color-border);
		color: var(--color-muted);
		cursor: pointer;
		font-size: 0.6rem;
		padding: 0.15rem 0.4rem;
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
</style>
