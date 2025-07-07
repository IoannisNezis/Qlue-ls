<script lang="ts">
    let { version } = $props();
</script>

<nav class="border-gray-200 bg-gray-50 dark:border-gray-700 dark:bg-gray-800">
    <div class="mx-auto flex max-w-(--breakpoint-xl) flex-wrap items-center justify-between p-4">
        <a href="/" class="flex items-center space-x-3 rtl:space-x-reverse">
            <img src="favicon.png" class="logo h-10" alt="Logo" />
            <div class="self-center">
                <span class="text-2xl font-semibold whitespace-nowrap dark:text-[#d79921]">
                    Qlue-ls
                </span>
                {#if version}
                    <span class="font-mono text-sm whitespace-nowrap dark:text-[#d79921]">
                        v{version}
                    </span>
                {/if}
            </div>
        </a>
        <div>
            <a
                href="https://github.com/IoannisNezis/sparql-language-server"
                class="flex flex-row p-2 text-white hover:text-gray-400"
            >
                <img class="h-6 w-6" src="/Git-Icon-White.svg" alt="" />
                <span class="ms-4"> IoannisNezis/Qlue-ls </span>
            </a>
        </div>
    </div>
</nav>
