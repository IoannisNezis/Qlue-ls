<script lang="ts">
    import { type editor } from 'monaco-editor';
    interface Props {
        markers: editor.IMarker[];
    }
    let { markers }: Props = $props();

    // const Hint = 1;
    const Info = 2;
    const Warning = 4;
    const Error = 8;
    let errorCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Error).length
    );
    let infoCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Info).length
    );
    let warningCount = $derived(
        markers.filter((item: editor.IMarker) => item.severity == Warning).length
    );
</script>

<div id="diagnosticOverview" class="grid shrink grid-cols-3 gap-2 border-s border-gray-600 px-3">
    <div class="flex flex-row">
        <img class="w-6 text-white" src="error.svg" alt="Errors:" />
        {errorCount}
    </div>
    <div class="flex flex-row">
        <img class="w-6 text-yellow-600" src="warning.svg" alt="Warnings:" />
        {warningCount}
    </div>
    <div class="flex flex-row">
        <img class="w-6 text-white" src="info.svg" alt="Infos:" />
        {infoCount}
    </div>
</div>
