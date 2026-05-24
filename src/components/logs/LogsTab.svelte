<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { logsStore } from "$lib/stores/logs.svelte";
  import LogFilter from "./LogFilter.svelte";
  import LogList from "./LogList.svelte";

  let loading = $state(false);
  let clearing = $state(false);

  onMount(async () => {
    loading = true;
    try {
      const entries = await api.listLogEntries(null, 500, 0);
      logsStore.setEntries(entries);
    } finally {
      loading = false;
    }
  });

  async function clearLogs() {
    clearing = true;
    try {
      await api.clearLogEntries();
      logsStore.clear();
    } finally {
      clearing = false;
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-3 py-2 border-b border-gray-700 shrink-0">
    <LogFilter
      filter={logsStore.levelFilter}
      onFilterChange={(f) => logsStore.setFilter(f)}
    />
    <button
      class="px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded hover:bg-gray-600 disabled:opacity-50"
      onclick={clearLogs}
      disabled={clearing}
    >
      Clear
    </button>
  </div>
  {#if loading}
    <div class="flex-1 flex items-center justify-center text-gray-500">Loading…</div>
  {:else}
    <div class="flex-1 overflow-hidden">
      <LogList entries={logsStore.filteredEntries} />
    </div>
  {/if}
</div>
