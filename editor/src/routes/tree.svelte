<script lang="ts">
    import { get_parse_tree } from 'll-sparql-parser';
    interface Node {
        kind: string;
        type: 'node';
        children: NodeOrToken[];
    }
    interface Token {
        kind: string;
        type: 'token';
        text: string;
    }
    type NodeOrToken = Node | Token;

    let { input } = $props();
    let parseTree = $derived(get_parse_tree(input));
</script>

{#snippet renderLeave(leave: Token)}
    <div>
        <span>
            {leave.kind}:
        </span>
        <span class="w-min text-red-400">
            {leave.text}
        </span>
    </div>
{/snippet}

{#snippet renderTree(tree: Node)}
    <span>
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
    class="overflow-y-auto overflow-x-hidden border-l border-gray-700 p-2 text-white"
>
    {@render renderTree(parseTree)}
</div>
