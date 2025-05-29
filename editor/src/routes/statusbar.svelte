<script lang="ts">
    import { type editor } from 'monaco-editor';
    import Markers from './markers.svelte';

    interface Props {
        markers: editor.IMarker[];
    }
    let { markers }: Props = $props();
</script>

<div class="flex w-full justify-end border-t border-gray-700 py-1 text-gray-300">
    <Markers {markers}></Markers>
</div>
