<script lang="ts">
	import { onMount } from 'svelte';
	import maplibregl from 'maplibre-gl';
	import {
		editorSelectedLocation,
		editorLocations,
		editorNpcs,
		editorSnapshot,
		editorDirty,
		editorValidation,
		editorSelectedLocationId
	} from '../../stores/editor';
	import { editorUpdateLocations, editorSave } from '$lib/editor-ipc';
	import type { GeoKind, LocationData } from '$lib/editor-types';
	import { getUiConfig } from '$lib/ipc';
	import { buildStyle, readThemeColors } from '$lib/map/style';
	import type { TileSource } from '$lib/types';

	let mapContainer: HTMLDivElement | undefined;
	let map: maplibregl.Map | null = null;
	let mapLoaded = false;
	let activeTile: TileSource | undefined;

	$: loc = $editorSelectedLocation;
	$: locations = $editorLocations;
	$: npcs = $editorNpcs;
	$: selectedId = $editorSelectedLocationId;

	function locationName(id: number): string {
		return locations.find((l) => l.id === id)?.name ?? `#${id}`;
	}

	function npcName(id: number): string {
		return npcs.find((n) => n.id === id)?.name ?? `#${id}`;
	}

	function moveLatLon(lat: number, lon: number, northM: number, eastM: number) {
		const dLat = northM / 111_320;
		const cosLat = Math.max(0.2, Math.cos((lat * Math.PI) / 180));
		const dLon = eastM / (111_320 * cosLat);
		return { lat: lat + dLat, lon: lon + dLon };
	}

	async function persistLocations(nextLocations: LocationData[]) {
		const report = await editorUpdateLocations(nextLocations);
		editorSnapshot.update((s) => {
			if (!s) return s;
			return { ...s, locations: nextLocations, validation: report };
		});
		editorValidation.set(report);
		editorDirty.set(true);
	}

	async function updateSelectedLocation(mutator: (location: LocationData) => LocationData) {
		if (!$editorSnapshot || !loc) return;
		const nextLocations = $editorSnapshot.locations.map((l) => (l.id === loc.id ? mutator(l) : l));
		try {
			await persistLocations(nextLocations);
		} catch (e) {
			console.error('Failed to update location:', e);
		}
	}

	async function handleFieldChange(field: string, value: unknown) {
		await updateSelectedLocation((current) => ({ ...current, [field]: value }));
	}

	async function setCoordinateMode(mode: 'absolute' | 'relative') {
		if (!loc) return;
		if (mode === 'absolute') {
			await handleFieldChange('relative_to', null);
			return;
		}
		const anchorCandidate = locations.find((l) => l.id !== loc.id);
		if (!anchorCandidate) return;
		await handleFieldChange('relative_to', {
			anchor: anchorCandidate.id,
			dnorth_m: 0,
			deast_m: 0
		});
	}

	async function applyRelativeField(field: 'anchor' | 'dnorth_m' | 'deast_m', raw: string) {
		if (!loc) return;
		const rel = loc.relative_to ?? { anchor: loc.id, dnorth_m: 0, deast_m: 0 };
		const value = field === 'anchor' ? Number(raw) : Number.parseFloat(raw);
		if (Number.isNaN(value)) return;
		await handleFieldChange('relative_to', { ...rel, [field]: value });
	}

	async function nudgeSelected(northM: number, eastM: number) {
		if (!loc) return;
		if (loc.relative_to) {
			await handleFieldChange('relative_to', {
				...loc.relative_to,
				dnorth_m: loc.relative_to.dnorth_m + northM,
				deast_m: loc.relative_to.deast_m + eastM
			});
			return;
		}
		const moved = moveLatLon(loc.lat, loc.lon, northM, eastM);
		await updateSelectedLocation((current) => ({ ...current, ...moved }));
	}

	async function toggleConnection(targetId: number) {
		if (!$editorSnapshot || !loc || targetId === loc.id) return;
		const source = loc;
		const hasConnection = source.connections.some((c) => c.target === targetId);
		const nextLocations = $editorSnapshot.locations.map((entry) => {
			if (entry.id === source.id) {
				const connections = hasConnection
					? entry.connections.filter((c) => c.target !== targetId)
					: [...entry.connections, { target: targetId, path_description: 'an old lane between settlements' }];
				return { ...entry, connections };
			}
			if (entry.id === targetId) {
				const reverseHas = entry.connections.some((c) => c.target === source.id);
				const connections = hasConnection
					? entry.connections.filter((c) => c.target !== source.id)
					: reverseHas
						? entry.connections
						: [...entry.connections, { target: source.id, path_description: 'an old lane between settlements' }];
				return { ...entry, connections };
			}
			return entry;
		});
		try {
			await persistLocations(nextLocations);
		} catch (e) {
			console.error('Failed to toggle connection:', e);
		}
	}

	async function handleSave() {
		try {
			const result = await editorSave(['world']);
			editorValidation.set(result.validation);
			if (result.saved) editorDirty.set(false);
		} catch (e) {
			console.error('Failed to save:', e);
		}
	}

	function refreshMapData() {
		if (!map || !mapLoaded) return;
		const features = locations.map((entry) => ({
			type: 'Feature' as const,
			properties: {
				id: entry.id,
				name: entry.name,
				selected: entry.id === selectedId ? 1 : 0,
				relative: entry.relative_to ? 1 : 0
			},
			geometry: { type: 'Point' as const, coordinates: [entry.lon, entry.lat] }
		}));
		const edgeFeatures = [];
		for (const entry of locations) {
			for (const conn of entry.connections) {
				if (entry.id > conn.target) continue;
				const target = locations.find((loc) => loc.id === conn.target);
				if (!target) continue;
				edgeFeatures.push({
					type: 'Feature' as const,
					properties: { a: entry.id, b: target.id },
					geometry: {
						type: 'LineString' as const,
						coordinates: [
							[entry.lon, entry.lat],
							[target.lon, target.lat]
						]
					}
				});
			}
		}
		(map.getSource('editor-locations') as maplibregl.GeoJSONSource)?.setData({
			type: 'FeatureCollection',
			features
		});
		(map.getSource('editor-edges') as maplibregl.GeoJSONSource)?.setData({
			type: 'FeatureCollection',
			features: edgeFeatures
		});
		if (loc) map.easeTo({ center: [loc.lon, loc.lat], duration: 250 });
	}

	onMount(() => {
		if (!mapContainer) return;
		let disposed = false;
		void (async () => {
			try {
				const cfg = await getUiConfig();
				activeTile =
					cfg.tile_sources.find((t) => t.id === cfg.active_tile_source) ?? cfg.tile_sources[0];
			} catch {
				activeTile = undefined;
			}
			if (disposed) return;
			map = new maplibregl.Map({
				container: mapContainer!,
				style: buildStyle('full', readThemeColors(), activeTile),
				center: [-8.0, 53.5],
				zoom: 12
			});
			map.addControl(new maplibregl.NavigationControl({ showCompass: false }), 'top-right');
			map.on('load', () => {
				mapLoaded = true;
				map?.addSource('editor-locations', {
					type: 'geojson',
					data: { type: 'FeatureCollection', features: [] }
				});
				map?.addSource('editor-edges', {
					type: 'geojson',
					data: { type: 'FeatureCollection', features: [] }
				});
				map?.addLayer({
				id: 'editor-edges',
				type: 'line',
				source: 'editor-edges',
				paint: { 'line-color': '#8f7e56', 'line-width': 2, 'line-opacity': 0.85 }
			});
				map?.addLayer({
				id: 'editor-locations',
				type: 'circle',
				source: 'editor-locations',
				paint: {
					'circle-radius': ['case', ['==', ['get', 'selected'], 1], 8, 5],
					'circle-color': [
						'case',
						['==', ['get', 'selected'], 1], '#f4cf75',
						['==', ['get', 'relative'], 1], '#7dd7ff',
						'#8f7e56'
					],
					'circle-stroke-width': 1.2,
					'circle-stroke-color': '#1a140a'
				}
			});
				map?.on('click', 'editor-locations', async (event) => {
				const rawId = event.features?.[0]?.properties?.id;
				const id = typeof rawId === 'number' ? rawId : Number(rawId);
				if (Number.isNaN(id)) return;
				if (selectedId && selectedId !== id && (event.originalEvent as MouseEvent).shiftKey) {
					await toggleConnection(id);
					return;
				}
				editorSelectedLocationId.set(id);
				});
				refreshMapData();
			});

			let dragging = false;
			let dragLat = 0;
			let dragLon = 0;
			map.on('mousedown', 'editor-locations', () => {
			dragging = true;
			if (loc) {
				dragLat = loc.lat;
				dragLon = loc.lon;
			}
			map?.dragPan.disable();
			});
			map.on('mousemove', (event) => {
			if (!dragging || !loc) return;
			dragLat = event.lngLat.lat;
			dragLon = event.lngLat.lng;
			const previewFeatures = locations.map((entry) => ({
				type: 'Feature' as const,
				properties: {
					id: entry.id,
					name: entry.name,
					selected: entry.id === selectedId ? 1 : 0,
					relative: entry.relative_to ? 1 : 0
				},
				geometry: {
					type: 'Point' as const,
					coordinates: entry.id === loc.id ? [dragLon, dragLat] : [entry.lon, entry.lat]
				}
			}));
			(map?.getSource('editor-locations') as maplibregl.GeoJSONSource)?.setData({
				type: 'FeatureCollection',
				features: previewFeatures
			});
			});
			map.on('mouseup', async () => {
			if (dragging && loc) {
				await updateSelectedLocation((current) => ({ ...current, lat: dragLat, lon: dragLon }));
			}
			dragging = false;
			map?.dragPan.enable();
			});
		})();

		return () => {
			disposed = true;
			map?.remove();
			map = null;
		};
	});

	$: refreshMapData();
</script>

<div class="loc-detail">
	{#if loc}
		<div class="detail-header">
			<h3 class="detail-title">{loc.name}</h3>
			<button class="save-btn" on:click={handleSave} disabled={!$editorDirty}>Save World</button>
		</div>

		<div class="detail-scroll">
			<section class="section">
				<h4 class="section-label">Map Designer</h4>
				<div class="map-frame" bind:this={mapContainer}></div>
				<p class="field-hint">Click to select, drag selected point to move. Shift-click another point to toggle a bidirectional link.</p>
			</section>

			<section class="section">
				<h4 class="section-label">Identity</h4>
				<div class="field-row">
					<label class="field-label">Name</label>
					<input
						class="field-input"
						type="text"
						value={loc.name}
						on:change={(e) => handleFieldChange('name', e.currentTarget.value)}
					/>
				</div>
				<div class="field-row">
					<label class="field-label">Indoor</label>
					<input
						type="checkbox"
						checked={loc.indoor}
						on:change={(e) => handleFieldChange('indoor', e.currentTarget.checked)}
					/>
				</div>
				<div class="field-row">
					<label class="field-label">Public</label>
					<input
						type="checkbox"
						checked={loc.public}
						on:change={(e) => handleFieldChange('public', e.currentTarget.checked)}
					/>
				</div>
			</section>

			<section class="section">
				<h4 class="section-label">Coordinates</h4>
				<div class="field-row">
					<label class="field-label">Geo kind</label>
					<select
						class="field-input"
						value={loc.geo_kind ?? 'fictional'}
						on:change={(e) => handleFieldChange('geo_kind', e.currentTarget.value as GeoKind)}
					>
						<option value="real">Real</option>
						<option value="manual">Manual</option>
						<option value="fictional">Fictional</option>
					</select>
				</div>
				<div class="field-row">
					<label class="field-label">Mode</label>
					<select
						class="field-input"
						value={loc.relative_to ? 'relative' : 'absolute'}
						on:change={(e) => setCoordinateMode(e.currentTarget.value as 'absolute' | 'relative')}
					>
						<option value="absolute">Absolute</option>
						<option value="relative">Relative</option>
					</select>
				</div>
				{#if loc.relative_to}
					<div class="field-row">
						<label class="field-label">Anchor</label>
						<select
							class="field-input"
							value={loc.relative_to.anchor}
							on:change={(e) => applyRelativeField('anchor', e.currentTarget.value)}
						>
							{#each locations.filter((l) => l.id !== loc.id) as option}
								<option value={option.id}>{option.name}</option>
							{/each}
						</select>
					</div>
					<div class="field-row">
						<label class="field-label">dNorth m</label>
						<input
							class="field-input short"
							type="number"
							step="1"
							value={loc.relative_to.dnorth_m}
							on:change={(e) => applyRelativeField('dnorth_m', e.currentTarget.value)}
						/>
						<label class="field-label">dEast m</label>
						<input
							class="field-input short"
							type="number"
							step="1"
							value={loc.relative_to.deast_m}
							on:change={(e) => applyRelativeField('deast_m', e.currentTarget.value)}
						/>
					</div>
				{:else}
					<div class="field-row">
						<label class="field-label">Lat</label>
						<input
							class="field-input short"
							type="number"
							step="0.00001"
							value={loc.lat}
							on:change={(e) => handleFieldChange('lat', parseFloat(e.currentTarget.value))}
						/>
						<label class="field-label">Lon</label>
						<input
							class="field-input short"
							type="number"
							step="0.00001"
							value={loc.lon}
							on:change={(e) => handleFieldChange('lon', parseFloat(e.currentTarget.value))}
						/>
					</div>
				{/if}
				<div class="field-row">
					<label class="field-label">Geo source</label>
					<input
						class="field-input"
						type="text"
						value={loc.geo_source ?? ''}
						on:change={(e) => handleFieldChange('geo_source', e.currentTarget.value || null)}
					/>
				</div>
				<div class="nudge-row">
					<button class="nudge-btn" on:click={() => nudgeSelected(100, 0)}>N +100m</button>
					<button class="nudge-btn" on:click={() => nudgeSelected(-100, 0)}>S +100m</button>
					<button class="nudge-btn" on:click={() => nudgeSelected(0, 100)}>E +100m</button>
					<button class="nudge-btn" on:click={() => nudgeSelected(0, -100)}>W +100m</button>
				</div>
			</section>

			<section class="section">
				<h4 class="section-label">Connections ({loc.connections.length})</h4>
				{#each loc.connections as conn}
					<div class="conn-row">
						<span class="conn-target">{locationName(conn.target)}</span>
						<span class="conn-desc">{conn.path_description}</span>
						<button class="nudge-btn" on:click={() => toggleConnection(conn.target)}>Remove</button>
					</div>
				{/each}
			</section>

			<section class="section">
				<h4 class="section-label">Description Template</h4>
				<textarea
					class="field-textarea tall"
					value={loc.description_template}
					on:change={(e) => handleFieldChange('description_template', e.currentTarget.value)}
				></textarea>
				<p class="field-hint">Placeholders: {'{time}'}, {'{weather}'}, {'{npcs_present}'}</p>
			</section>

			<section class="section">
				<h4 class="section-label">Associated NPCs</h4>
				{#each loc.associated_npcs as npc_id}
					<span class="assoc-npc">{npcName(npc_id)}</span>
				{/each}
				{#if loc.associated_npcs.length === 0}
					<p class="empty-note">None</p>
				{/if}
			</section>
		</div>
	{:else}
		<div class="empty-state">
			<p>Select a location from the list to edit.</p>
		</div>
	{/if}
</div>

<style>
	.loc-detail {
		flex: 1;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.detail-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.5rem 0.8rem;
		border-bottom: 1px solid var(--color-border);
	}

	.detail-title {
		font-family: 'Cinzel', serif;
		font-size: 0.95rem;
		margin: 0;
		color: var(--color-accent);
	}

	.save-btn,
	.nudge-btn {
		padding: 0.25rem 0.6rem;
		border: 1px solid var(--color-accent);
		border-radius: 3px;
		background: none;
		color: var(--color-accent);
		font-size: 0.7rem;
		font-family: 'IM Fell English', serif;
		cursor: pointer;
	}
	.save-btn:hover:not(:disabled),
	.nudge-btn:hover {
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
	}

	.save-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.detail-scroll {
		flex: 1;
		overflow-y: auto;
		padding: 0.5rem 0.8rem;
	}

	.section {
		margin-bottom: 1rem;
	}

	.section-label {
		font-size: 0.7rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--color-muted);
		margin: 0 0 0.3rem;
		border-bottom: 1px solid var(--color-border);
		padding-bottom: 0.15rem;
	}

	.field-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.25rem;
	}

	.field-label {
		font-size: 0.72rem;
		color: var(--color-muted);
		min-width: 70px;
		flex-shrink: 0;
	}

	.field-input {
		flex: 1;
		padding: 0.2rem 0.35rem;
		border: 1px solid var(--color-border);
		border-radius: 3px;
		background: var(--color-input-bg);
		color: var(--color-fg);
		font-size: 0.75rem;
		font-family: 'IM Fell English', serif;
	}
	.field-input.short {
		max-width: 100px;
	}

	.map-frame {
		height: 320px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		overflow: hidden;
	}

	.nudge-row {
		display: flex;
		gap: 0.4rem;
		flex-wrap: wrap;
		margin-top: 0.4rem;
	}

	.field-textarea {
		width: 100%;
		min-height: 3rem;
		padding: 0.2rem 0.35rem;
		border: 1px solid var(--color-border);
		border-radius: 3px;
		background: var(--color-input-bg);
		color: var(--color-fg);
		font-size: 0.75rem;
		font-family: 'IM Fell English', serif;
		resize: vertical;
		box-sizing: border-box;
	}
	.field-textarea.tall {
		min-height: 5rem;
	}

	.field-hint {
		font-size: 0.6rem;
		color: var(--color-muted);
		margin: 0.15rem 0 0;
	}

	.conn-row {
		display: flex;
		gap: 0.5rem;
		align-items: center;
		padding: 0.15rem 0;
		font-size: 0.75rem;
		border-bottom: 1px solid color-mix(in srgb, var(--color-border) 50%, transparent);
	}

	.conn-target {
		font-weight: 600;
		min-width: 120px;
	}

	.conn-desc {
		font-style: italic;
		color: var(--color-muted);
		font-size: 0.7rem;
		flex: 1;
	}

	.assoc-npc {
		display: inline-block;
		font-size: 0.7rem;
		padding: 0.1rem 0.3rem;
		margin: 0.1rem;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-accent) 12%, transparent);
		color: var(--color-accent);
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--color-muted);
		font-size: 0.85rem;
	}

	.empty-note {
		color: var(--color-muted);
		font-size: 0.7rem;
		font-style: italic;
	}
</style>
