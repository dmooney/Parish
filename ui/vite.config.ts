import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit(), svelteTesting()],
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		fs: {
			allow: ['.']
		}
	},
	test: {
		include: ['src/**/*.test.ts'],
		globals: true,
		environment: 'jsdom',
		// jsdom 29 throws SecurityError when accessing localStorage on opaque
		// origins, so set a real URL so the storage APIs are available.
		environmentOptions: {
			jsdom: {
				url: 'http://localhost/'
			}
		},
		// Node.js v25 ships an experimental built-in `localStorage` global
		// that shadows jsdom's real implementation when `globals: true`.
		// Disable Node's stub so jsdom's wins inside vitest workers.
		execArgv: ['--no-experimental-webstorage'],
		setupFiles: ['src/test-setup.ts']
	}
});
