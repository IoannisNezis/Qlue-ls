<script lang="ts">
    import { backends, type Backend } from '$lib/backends';
    interface Props {
        backend: Backend;
    }
    let { backend = $bindable() }: Props = $props();
    let modal: HTMLElement;
</script>

<button class="hover:cursor-pointer" onclick={() => modal.showModal()}>{backend.name}</button>
<dialog bind:this={modal} id="my_modal_1" class="modal">
    <div class="modal-box max-w-none md:w-3/4 lg:w-2/3">
        <select bind:value={backend} class="select">
            {#each backends as backendConf}
                <option value={backendConf.backend}>{backendConf.backend.name}</option>
            {/each}
        </select>
        <div class="divider"></div>
        <div class="grid grid-cols-3">
            <div>slug:</div>
            <div class="col-span-2">
                {backend.slug}
            </div>
            <div>url:</div>
            <div class="col-span-2">
                {backend.url}
            </div>
            <div>health-check:</div>
            <div class="col-span-2">
                {backend.healthCheckUrl}
            </div>
        </div>
        <div class="modal-action">
            <form method="dialog">
                <button class="btn">Close</button>
            </form>
        </div>
    </div>
</dialog>
