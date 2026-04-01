import { writable, get } from 'svelte/store';
import type { ThemePalette } from '$lib/types';

const defaultPalette: ThemePalette = {
	bg: '#fff5dc',
	fg: '#32230f',
	accent: '#b48232',
	panel_bg: '#faf0d7',
	input_bg: '#f5ebd2',
	border: '#d2be96',
	muted: '#78643c',
	is_dark: false
};

/** Whether a wipe transition is currently in progress. */
export const wipeActive = writable(false);

/** The background color for the wipe overlay (the incoming theme's bg). */
export const wipeBg = writable('#fff5dc');

function createPaletteStore() {
	const { subscribe, set } = writable<ThemePalette>(defaultPalette);

	let prevIsDark: boolean | null = null;
	let wipeTimeout: ReturnType<typeof setTimeout> | null = null;

	function applyVars(palette: ThemePalette) {
		if (typeof document !== 'undefined') {
			const root = document.documentElement;
			root.style.setProperty('--color-bg', palette.bg);
			root.style.setProperty('--color-fg', palette.fg);
			root.style.setProperty('--color-accent', palette.accent);
			root.style.setProperty('--color-panel-bg', palette.panel_bg);
			root.style.setProperty('--color-input-bg', palette.input_bg);
			root.style.setProperty('--color-border', palette.border);
			root.style.setProperty('--color-muted', palette.muted);
		}
	}

	function apply(palette: ThemePalette) {
		const isDark = palette.is_dark ?? false;

		// Detect a light↔dark transition
		if (prevIsDark !== null && isDark !== prevIsDark) {
			// Trigger the wipe transition
			wipeBg.set(palette.bg);
			wipeActive.set(true);

			// Clear any pending timeout from a previous wipe
			if (wipeTimeout) clearTimeout(wipeTimeout);

			// Apply the new palette midway through the wipe (300ms)
			wipeTimeout = setTimeout(() => {
				set(palette);
				applyVars(palette);
			}, 300);

			// Remove the wipe overlay after the animation completes (600ms)
			setTimeout(() => {
				wipeActive.set(false);
				wipeTimeout = null;
			}, 600);
		} else {
			// No transition needed — apply immediately
			set(palette);
			applyVars(palette);
		}

		prevIsDark = isDark;
	}

	// Apply defaults immediately
	apply(defaultPalette);

	return { subscribe, apply };
}

export const palette = createPaletteStore();
