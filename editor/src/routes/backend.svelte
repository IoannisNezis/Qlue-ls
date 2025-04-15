<script lang="ts">
    import { backends, type Backend, type BackendConf } from '$lib/backends';
    import { LanguageClientWrapper } from 'monaco-editor-wrapper';
    import BackendDetail from './backendDetail.svelte';
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

    function checkAvailibility() {
        languageClientWrapper
            .getLanguageClient()!
            .sendRequest('qlueLs/pingBackend', backend!.name)
            .then((response: SetBackendResponse) => {
                backend_state.availibility = response.availible ? 'up' : 'down';
            })
            .catch((err) => {
                console.error(err);
            });
    }

    function refreshAvailibility() {
        checkAvailibility();
        setTimeout(refreshAvailibility, 10000);
    }

    $effect(() => {
        if (languageClientWrapper) {
            backends.forEach(addBackend);
            refreshAvailibility();
        }
    });

    $effect(() => {
        if (backend && languageClientWrapper) {
            checkAvailibility();

            languageClientWrapper
                .getLanguageClient()!
                .sendRequest('qlueLs/updateDefaultBackend', backend.name)
                .catch((err) => {
                    console.error(err);
                });
        }
    });
</script>

<div class="flex flex-row border-s border-gray-600 px-3">
    {#if backend_state.availibility == 'unknown'}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 200" class="size-6"
            ><radialGradient
                id="a5"
                cx=".66"
                fx=".66"
                cy=".3125"
                fy=".3125"
                gradientTransform="scale(1.5)"
                ><stop offset="0" stop-color="#B5E1FF"></stop><stop
                    offset=".3"
                    stop-color="#B5E1FF"
                    stop-opacity=".9"
                ></stop><stop offset=".6" stop-color="#B5E1FF" stop-opacity=".6"></stop><stop
                    offset=".8"
                    stop-color="#B5E1FF"
                    stop-opacity=".3"
                ></stop><stop offset="1" stop-color="#B5E1FF" stop-opacity="0"
                ></stop></radialGradient
            ><circle
                transform-origin="center"
                fill="none"
                stroke="url(#a5)"
                stroke-width="25"
                stroke-linecap="round"
                stroke-dasharray="200 1000"
                stroke-dashoffset="0"
                cx="100"
                cy="100"
                r="70"
                ><animateTransform
                    type="rotate"
                    attributeName="transform"
                    calcMode="spline"
                    dur="2.6"
                    values="360;0"
                    keyTimes="0;1"
                    keySplines="0 0 1 1"
                    repeatCount="indefinite"
                ></animateTransform></circle
            ><circle
                transform-origin="center"
                fill="none"
                opacity=".2"
                stroke="#B5E1FF"
                stroke-width="25"
                stroke-linecap="round"
                cx="100"
                cy="100"
                r="70"
            ></circle></svg
        >
    {:else if backend_state.availibility == 'down'}
        <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-6 text-red-600"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z"
            />
        </svg>
    {:else}
        <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-6 text-green-400"
        >
            <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M9 12.75 11.25 15 15 9.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
            />
        </svg>
    {/if}
    <BackendDetail bind:backend />
</div>
