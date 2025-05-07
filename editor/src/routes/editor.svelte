<script lang="ts">
    import { onDestroy, onMount } from 'svelte';
    import Statusbar from './statusbar.svelte';
    import type {
        LanguageClientWrapper,
        MonacoEditorLanguageClientWrapper
    } from 'monaco-editor-wrapper';
    import type { editor } from 'monaco-editor';
    import { backends } from '$lib/backends';

    let { ready = $bindable() } = $props();

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
        ready = true;
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
        monaco.editor.addCommand({
            id: 'triggerNewCompletion',
            run: () => {
                editor.trigger('editor', 'editor.action.triggerSuggest', {});
            }
        });

        monaco.editor.addCommand({
            id: 'jumpToNextSnippetPlaceholder',
            run: () => {
                languageClientWrapper
                    ?.getLanguageClient()!
                    .sendRequest('textDocument/formatting', {
                        textDocument: { uri: editor.getModel()?.uri.toString() },
                        options: {
                            tabSize: 2,
                            insertSpaces: true
                        }
                    })
                    .then((response) => {
                        const edits = response.map((edit) => {
                            console.log(edit);
                            return {
                                range: {
                                    startLineNumber: edit.range.start.line + 1,
                                    startColumn: edit.range.start.character + 1,
                                    endLineNumber: edit.range.end.line + 1,
                                    endColumn: edit.range.end.character + 1
                                },
                                text: edit.newText
                            };
                        });
                        editor.getModel()!.applyEdits(edits);

                        const cursorPosition = editor.getPosition();
                        languageClientWrapper
                            ?.getLanguageClient()!
                            .sendRequest('qlueLs/jump', {
                                textDocument: { uri: editor.getModel()?.uri.toString() },
                                position: {
                                    line: cursorPosition?.lineNumber - 1,
                                    character: cursorPosition?.column - 1
                                }
                            })
                            .then((response) => {
                                if (response) {
                                    const newCursorPosition = {
                                        lineNumber: response.position.line + 1,
                                        column: response.position.character + 1
                                    };
                                    if (response.insertAfter) {
                                        editor.executeEdits('jumpToNextSnippetPlaceholder', [
                                            {
                                                range: new monaco.Range(
                                                    newCursorPosition.lineNumber,
                                                    newCursorPosition.column,
                                                    newCursorPosition.lineNumber,
                                                    newCursorPosition.column
                                                ),
                                                text: response.insertAfter
                                            }
                                        ]);
                                    }
                                    editor.setPosition(
                                        newCursorPosition,
                                        'jumpToNextSnippetPlaceholder'
                                    );
                                    if (response.insertBefore) {
                                        editor.getModel()?.applyEdits([
                                            {
                                                range: new monaco.Range(
                                                    newCursorPosition.lineNumber,
                                                    newCursorPosition.column,
                                                    newCursorPosition.lineNumber,
                                                    newCursorPosition.column
                                                ),
                                                text: response.insertBefore
                                            }
                                        ]);
                                    }
                                    editor.trigger('editor', 'editor.action.triggerSuggest', {});
                                }
                            });
                    });
                editor.trigger('jumpToNextSnippetPlaceholder', 'editor.action.formatDocument', {});
                console.log('jump to next location');
            }
        });
        monaco.editor.addKeybindingRule({
            command: 'jumpToNextSnippetPlaceholder',
            keybinding: monaco.KeyMod.Alt | monaco.KeyCode.KeyN
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
