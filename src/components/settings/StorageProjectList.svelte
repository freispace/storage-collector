<script lang="ts">
  import { onMount } from "svelte";
  import { api, type FolderConfig, type StorageProjectItem } from "$lib/api";
  import { namesStore } from "$lib/stores/names.svelte";
  import StorageProjectRow from "./StorageProjectRow.svelte";
  import LoadingText from "../general/loadingText.svelte";

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

    if (page === 1) {
      // Load cached names immediately, then sync in background for any new ones
      namesStore.load();
      api
        .syncEntityNames()
        .catch(() => {})
        .then(() => namesStore.load());
    }
  }

  async function refreshConfigs() {
    folderConfigs = await api.listFolderConfigs();
  }

  function getConfigsForItem(item: StorageProjectItem): FolderConfig[] {
    return folderConfigs.filter(
      (fc) =>
        fc.storage_id === item.storage_id && fc.project_id === item.project_id,
    );
  }

  onMount(() => {
    loadPage(1);
  });
</script>

<div class="h-full overflow-y-auto space-y-2">
  <div class="flex items-center justify-between">
    <button class="btn btn-ghost" onclick={() => loadPage(1)}>
      <svg xmlns="http://www.w3.org/2000/svg" class="size-4" viewBox="0 0 24 24"
        ><path
          fill="currentColor"
          d="M12 20q-3.35 0-5.675-2.325T4 12t2.325-5.675T12 4q1.725 0 3.3.712T18 6.75V4h2v7h-7V9h4.2q-.8-1.4-2.187-2.2T12 6Q9.5 6 7.75 7.75T6 12t1.75 4.25T12 18q1.925 0 3.475-1.1T17.65 14h2.1q-.7 2.65-2.85 4.325T12 20"
        /></svg
      >
      <span>Refresh list</span>
    </button>
  </div>

  {#if loading}
    <div class="text-center py-4"><LoadingText /></div>
  {:else if error}
    <div class="text-sm text-red-400 bg-red-900/20 rounded p-2">{error}</div>
  {:else if items.length === 0}
    <div class="text-center text-gray-500 py-4 text-sm">
      No storage projects found
    </div>
  {:else}
    <div class="space-y-2">
      {#each items as item (item.id)}
        <StorageProjectRow
          {item}
          folderConfigs={getConfigsForItem(item)}
          {globalSchedule}
          names={namesStore.names}
          onConfigsChanged={refreshConfigs}
        />
      {/each}
    </div>

    {#if nextLink !== null}
      <button
        class="btn btn-block"
        onclick={() => loadPage(currentPage + 1, true)}
        disabled={loadingMore}
      >
        {loadingMore ? "Loading…" : "Load more"}
      </button>
    {/if}
  {/if}
</div>
