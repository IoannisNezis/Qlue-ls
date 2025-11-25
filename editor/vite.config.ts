import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';
import path from 'path';

export default defineConfig(({ mode }) => ({
    optimizeDeps: {
        include: [
            '@testing-library/react',
            'vscode/localExtensionHost',
            'vscode-textmate',
            'vscode-oniguruma'
        ]
    },
    resolve: {
      alias: mode === 'development' ? [
        { find: /^qlue-ls(\?.*)?$/, replacement: path.resolve(__dirname, '../pkg') + '$1' },
        { find: /^ll-sparql-parser(\?.*)?$/, replacement: path.resolve(__dirname, '../crates/parser/pkg') + '$1' }
      ] : [] 
    },
    server: {
        allowedHosts: true,
        fs: {
            strict: false
        }
    },
    worker: {
        format: 'es',
        plugins: () => [wasm()]
    },
    plugins: [sveltekit(), tailwindcss(), wasm()]
}));
