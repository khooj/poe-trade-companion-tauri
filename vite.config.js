import { sveltekit } from '@sveltejs/kit/vite';
// import { defineConfig } from 'vitest/config';

export default {
	plugins: [sveltekit()],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}']
	},
	clearScreen: false,
	server: {
		port: 5174,
		strictPort: true,
	},
	envPrefix: ["VITE_", "TAURI_"],
};
