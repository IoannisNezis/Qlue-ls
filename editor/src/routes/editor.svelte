<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Statusbar from './statusbar.svelte';
    import type {
        LanguageClientWrapper,
        MonacoEditorLanguageClientWrapper
    } from 'monaco-editor-wrapper';
    import type { editor } from 'monaco-editor';
    import { backends } from '$lib/backends';

    let editorContainer: HTMLElement;
    let wrapper: MonacoEditorLanguageClientWrapper | undefined;
    let languageClientWrapper: LanguageClientWrapper | undefined = $state();
    let markers: editor.IMarker[] = $state([]);
    let content = $state('SELECT * WHERE {\n  \n}');
    let cursorOffset = $state(0);
    let backend = $state(backends.find((backendConf) => backendConf.default)!.backend);
    // let backend = $state(backends[1].backend);

    onMount(async () => {
        const { MonacoEditorLanguageClientWrapper } = await import('monaco-editor-wrapper');
        const { buildWrapperConfig } = await import('$lib/config');
        const monaco = await import('monaco-editor');

        wrapper = new MonacoEditorLanguageClientWrapper();
        let wrapperConfig = await buildWrapperConfig(editorContainer, content);
        await wrapper.initAndStart(wrapperConfig);
        languageClientWrapper = wrapper.getLanguageClientWrapper('sparql');
        let editor = wrapper.getEditor()!;

        monaco.editor.onDidChangeMarkers(() => {
            markers = monaco.editor.getModelMarkers({});
        });
        editor.getModel()!.onDidChangeContent(() => {
            content = wrapper?.getEditor()!.getModel()!.getValue();
        });
        editor.onDidChangeCursorPosition((e) => {
            cursorOffset = wrapper?.getEditor()!.getModel()!.getOffsetAt(e.position);
        });
        wrapper.getEditor()!.addAction({
            id: 'Execute Query',
            label: 'Execute',
            keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter],
            contextMenuGroupId: 'navigation',
            contextMenuOrder: 1.5,
            run(editor, ...args) {
                const encoded_query = encodeURIComponent(editor.getModel()?.getValue()).replaceAll(
                    '%20',
                    '+'
                );
                window.open(
                    `https://qlever.cs.uni-freiburg.de/${backend.slug}/?query=${encoded_query}`
                );
            }
        });
    });

    onDestroy(() => {
        wrapper?.dispose(true);
    });

    let showTree = $state(false);
</script>

<div class="relative grid grid-cols-3">
    <div
        id="editor"
        class="container transition-all {showTree ? 'col-span-2' : 'col-span-3'}"
        bind:this={editorContainer}
    ></div>

    <!-- svelte-ignore a11y_consider_explicit_label -->
    <button
        onclick={() => (showTree = !showTree)}
        class="absolute top-2 right-2 rounded-sm bg-gray-700 px-2 py-2 font-bold text-white hover:bg-gray-600"
    >
        <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-5 transition duration-200 {showTree ? 'rotate-180' : ''}"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="m18.75 4.5-7.5 7.5 7.5 7.5m-6-15L5.25 12l7.5 7.5"
            />
        </svg>
    </button>
</div>
<Statusbar {languageClientWrapper} {markers} bind:backend></Statusbar>

<style>
    #editor {
        height: 60vh;
    }
</style>
