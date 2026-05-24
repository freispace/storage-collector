<script lang="ts">
  import { onMount } from "svelte";
  import { onLogEntry, type LogEntry } from "$lib/api";
  import { logsStore } from "$lib/stores/logs.svelte";
  import LogsTab from "./components/logs/LogsTab.svelte";

  onMount(async () => {
    const unlisten = await onLogEntry((entry: LogEntry) => {
      logsStore.addEntry(entry);
    });
    return unlisten;
  });
</script>

<div class="flex flex-col h-screen bg-gray-900 text-gray-100 text-sm select-none">
  <LogsTab />
</div>
