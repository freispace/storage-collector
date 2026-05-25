<script lang="ts">
  import { onMount } from "svelte";
  import { onLogEntry, type LogEntry } from "$lib/api";
  import { logsStore } from "$lib/stores/logs.svelte";
  import Logs from "./components/logs/Logs.svelte";

  onMount(() => {
    const unlistenPromise = onLogEntry((entry: LogEntry) => {
      logsStore.addEntry(entry);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });
</script>

<div class="flex flex-col h-screen bg-gray-800 text-gray-100 text-sm select-none">
  <Logs />
</div>
