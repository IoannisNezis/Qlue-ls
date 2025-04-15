import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite'
import importMetaUrlPlugin from '@codingame/esbuild-import-meta-url-plugin'
import wasm from "vite-plugin-wasm";

export default defineConfig(({ mode }) => ({
	optimizeDeps: {
		esbuildOptions: {
			plugins: [importMetaUrlPlugin]
		},
		include: [
			'@testing-library/react',
			'vscode/localExtensionHost',
			'vscode-textmate',
			'vscode-oniguruma'
		]
	},
	worker: {
		format: "es",
		plugins: () => [
			wasm(),
		]
	},
	plugins: [
		sveltekit(),
		tailwindcss(),
		wasm(),
	]
}));
