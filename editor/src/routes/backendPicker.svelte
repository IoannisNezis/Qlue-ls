<script lang="ts">
    import { backends, type Backend, type BackendConf } from '$lib/backends';
    import { LanguageClientWrapper } from 'monaco-editor-wrapper';
    interface Props {
        languageClientWrapper: LanguageClientWrapper;
        backend: Backend;
    }
    interface BackendState {
        backend: Backend;
        availibility: 'unknown' | 'up' | 'down';
    }

    interface SetBackendResponse {
        availible: boolean;
    }
    let { languageClientWrapper, backend = $bindable() }: Props = $props();

    let backend_state: BackendState = $state({
        backend: { ...backend },
        availibility: 'unknown'
    });

    function addBackend(conf: BackendConf) {
        languageClientWrapper
            .getLanguageClient()!
            .sendRequest('qlueLs/addBackend', conf)
            .catch((err) => {
                console.error(err);
            });
    }

    $effect(() => {
        if (languageClientWrapper) {
            backends.forEach(addBackend);
        }
    });

    $effect(() => {
        if (backend && languageClientWrapper) {
            languageClientWrapper
                .getLanguageClient()!
                .sendRequest('qlueLs/updateDefaultBackend', backend.name)
                .catch((err) => {
                    console.error(err);
                });
        }
    });
</script>

<select bind:value={backend} class="select">
    {#each backends as backendConf}
        <option value={backendConf.backend}>
            {backendConf.backend.name}
        </option>
    {/each}
</select>
