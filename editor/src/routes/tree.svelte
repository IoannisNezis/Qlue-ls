<script lang="ts">
    import { get_parse_tree } from 'll-sparql-parser';
    interface TreeElement {
        kind: string;
        active: boolean;
    }
    interface Node extends TreeElement {
        type: 'node';
        children: NodeOrToken[];
    }
    interface Token extends TreeElement {
        type: 'token';
        text: string;
    }
    type NodeOrToken = Node | Token;

    let { input, cursorOffset } = $props();
    let parseTree = $derived(get_parse_tree(input, cursorOffset));
</script>

{#snippet renderLeave(leave: Token)}
    <div>
        <span class={leave.active ? 'bg-yellow-400' : ''}>
            {leave.kind}:
        </span>
        <span class="w-min text-red-400">
            {leave.text}
        </span>
    </div>
{/snippet}

{#snippet renderTree(tree: Node)}
    <span class={tree.active ? 'bg-yellow-600' : ''}>
        {tree.kind}
    </span>
    <div class="ms-2 flex flex-col border-l ps-2">
        {#each tree.children as child}
            {#if child.type == 'node'}
                <span>
                    {@render renderTree(child)}
                </span>
            {:else}
                {@render renderLeave(child)}
            {/if}
        {/each}
    </div>
{/snippet}

<div
    id="treeContainer"
    style="height: 60vh;"
    class="overflow-auto border-l border-gray-700 p-2 text-white"
>
    {@render renderTree(parseTree)}
</div>
