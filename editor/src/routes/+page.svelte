<script>
    import Editor from './editor.svelte';
    import Header from './header.svelte';
    import LoadingAnimation from './loadingAnimation.svelte';

    let ready = $state(false);
    let version = $state('');
</script>

<Header {version}></Header>
<div class="container m-auto">
    {#if !ready}
        <div class="mt-10 grid h-[650px] place-content-center rounded-xs border-2 bg-[#282828] p-2">
            <div class="flex flex-col items-center gap-3">
                <LoadingAnimation></LoadingAnimation>
                <div class="p-2 text-3xl text-white">Loading Monaco Editor ...</div>
            </div>
        </div>
    {/if}
    <div class:hidden={!ready} class="mt-5">
        <Editor bind:ready bind:version></Editor>
    </div>
</div>
