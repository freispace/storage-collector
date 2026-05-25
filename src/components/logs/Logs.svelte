<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { logsStore } from "$lib/stores/logs.svelte";
  import { namesStore } from "$lib/stores/names.svelte";
  import { formatDate } from "$lib/utils";
  import LogFilter from "./LogFilter.svelte";
  import LogList from "./LogList.svelte";
  import LoadingText from "../general/loadingText.svelte";

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

  const UUID_RE =
    /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi;

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
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-4 text-gray-400"
          viewBox="0 0 24 24"
          ><path
            fill="currentColor"
            d="M21 7v14H3V3h14zm-9 11q1.25 0 2.125-.875T15 15t-.875-2.125T12 12t-2.125.875T9 15t.875 2.125T12 18m-6-8h9V6H6z"
          /></svg
        >
        <span>Export</span>
      </button>
      <button class="btn" onclick={clearLogs} disabled={clearing}>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-4 text-gray-400"
          viewBox="0 0 24 24"
          ><path
            d="M12.48 3L7.73 7.75L3 12.59a2 2 0 0 0 0 2.82l4.3 4.3A1 1 0 0 0 8 20h12v-2h-7l7.22-7.22a2 2 0 0 0 0-2.83L15.31 3a2 2 0 0 0-2.83 0zM8.41 18l-4-4l4.75-4.84l.74-.75l4.95 4.95l-4.56 4.56l-.07.08z"
            fill="currentColor"
          /></svg
        >
        <span>Clear</span>
      </button>
    </div>
  </div>
  {#if loading}
    <div
      class="flex-1 flex items-center justify-center text-gray-500 font-medium text-xl"
    >
      <LoadingText />
    </div>
  {:else}
    <div class="flex-1 overflow-hidden">
      <LogList entries={logsStore.filteredEntries} />
    </div>
  {/if}
</div>
