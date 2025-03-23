<script lang="ts">
    import { type editor } from 'monaco-editor';
    import { LanguageClientWrapper } from 'monaco-editor-wrapper';
    import Markers from './markers.svelte';
    import Backend from './backend.svelte';
    interface Props {
        languageClientWrapper: LanguageClientWrapper;
        markers: editor.IMarker[];
    }
    let { languageClientWrapper, markers }: Props = $props();
</script>

<div class="flex w-full justify-end border-t border-gray-700 py-1 text-gray-300">
    <Backend {languageClientWrapper}></Backend>
    <Markers {markers}></Markers>
</div>
