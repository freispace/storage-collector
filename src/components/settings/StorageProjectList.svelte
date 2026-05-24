<script lang="ts">
  import { onMount } from "svelte";
  import { api, type FolderConfig, type StorageProjectItem } from "$lib/api";
  import StorageProjectRow from "./StorageProjectRow.svelte";

  interface Props {
    globalSchedule: string;
  }

  let { globalSchedule }: Props = $props();

  let items = $state<StorageProjectItem[]>([]);
  let folderConfigs = $state<FolderConfig[]>([]);
  let nextLink = $state<string | null>(null);
  let currentPage = $state(1);
  let loading = $state(false);
  let loadingMore = $state(false);
  let error = $state<string | null>(null);

  async function loadPage(page: number, append = false) {
    if (page === 1) loading = true;
    else loadingMore = true;
    error = null;

    try {
      const [result, configs] = await Promise.all([
        api.fetchStorageProjectsPage(page),
        page === 1 ? api.listFolderConfigs() : Promise.resolve(folderConfigs),
      ]);

      items = append ? [...items, ...result.data] : result.data;
      folderConfigs = configs as FolderConfig[];
      nextLink = result.pagination_links?.next ?? null;
      currentPage = page;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
      loadingMore = false;
    }
  }

  async function refreshConfigs() {
    folderConfigs = await api.listFolderConfigs();
  }

  function getConfigsForItem(item: StorageProjectItem): FolderConfig[] {
    return folderConfigs.filter(
      (fc) => fc.storage_id === item.storage_id && fc.project_id === item.project_id
    );
  }

  onMount(() => { loadPage(1); });
</script>

<div class="space-y-2">
  <div class="flex items-center justify-between">
    <span class="text-xs font-medium text-gray-400 uppercase tracking-wide">
      Storage Projects
    </span>
    <button class="text-xs text-gray-500 hover:text-gray-300" onclick={() => loadPage(1)}>
      Refresh
    </button>
  </div>

  {#if loading}
    <div class="text-center text-gray-500 py-4 text-sm">Loading…</div>
  {:else if error}
    <div class="text-sm text-red-400 bg-red-900/20 rounded p-2">{error}</div>
  {:else if items.length === 0}
    <div class="text-center text-gray-500 py-4 text-sm">No storage projects found</div>
  {:else}
    <div class="space-y-2">
      {#each items as item (item.id)}
        <StorageProjectRow
          {item}
          folderConfigs={getConfigsForItem(item)}
          {globalSchedule}
          onConfigsChanged={refreshConfigs}
        />
      {/each}
    </div>

    {#if nextLink !== null}
      <button
        class="w-full py-2 text-sm text-gray-400 bg-gray-800 rounded hover:bg-gray-700
               disabled:opacity-50"
        onclick={() => loadPage(currentPage + 1, true)}
        disabled={loadingMore}
      >
        {loadingMore ? "Loading…" : "Load more"}
      </button>
    {/if}
  {/if}
</div>
