<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { logsStore } from "$lib/stores/logs.svelte";
  import { namesStore } from "$lib/stores/names.svelte";
  import { formatDate } from "$lib/utils";
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
    namesStore.load();
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

  const UUID_RE = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi;

  async function exportLogs() {
    const lines = logsStore.filteredEntries.map((e) => {
      const msg = e.message.replace(UUID_RE, (id) => {
        const name = namesStore.lookup(id);
        return name ? `${name} (${id})` : id;
      });
      return `[${formatDate(e.created_at)}] [${e.level.toUpperCase()}] ${msg}`;
    });

    const filter = logsStore.levelFilter ? `_${logsStore.levelFilter}` : "";
    await api.saveLogFile(lines.join("\n"), `logs${filter}.txt`);
  }
</script>

<div class="flex flex-col h-full">
  <div class="flex items-center justify-between px-3 py-2 shrink-0">
    <LogFilter
      filter={logsStore.levelFilter}
      onFilterChange={(f) => logsStore.setFilter(f)}
    />
    <div class="flex gap-2">
      <button
        class="btn"
        onclick={exportLogs}
        disabled={logsStore.filteredEntries.length === 0}
      >
        Export
      </button>
      <button
        class="btn"
        onclick={clearLogs}
        disabled={clearing}
      >
        Clear
      </button>
    </div>
  </div>
  {#if loading}
    <div class="flex-1 flex items-center justify-center text-gray-500 font-medium text-xl">
      <span class="loading loading-infinity loading-xl me-1"></span> Loading…
    </div>
  {:else}
    <div class="flex-1 overflow-hidden">
      <LogList entries={logsStore.filteredEntries} />
    </div>
  {/if}
</div>
