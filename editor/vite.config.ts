import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import wasm from 'vite-plugin-wasm';

export default defineConfig(({ mode }) => ({
    optimizeDeps: {
        include: [
            '@testing-library/react',
            'vscode/localExtensionHost',
            'vscode-textmate',
            'vscode-oniguruma'
        ]
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
