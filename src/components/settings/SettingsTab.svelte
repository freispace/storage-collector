<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import ApiKeyInput from "./ApiKeyInput.svelte";
  import GlobalSchedule from "./GlobalSchedule.svelte";
  import StorageProjectList from "./StorageProjectList.svelte";

  let running = $state(false);
  let runError = $state<string | null>(null);

  onMount(() => {
    settingsStore.load();
  });

  async function triggerAll() {
    running = true;
    runError = null;
    try {
      await api.triggerAll();
    } catch (e) {
      runError = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="h-full overflow-y-auto px-4 py-3 space-y-5">
  {#if !settingsStore.loaded}
    <div class="text-center text-gray-500 py-8">Loading settings…</div>
  {:else}
    <ApiKeyInput
      apiKey={settingsStore.apiKey}
      onChange={(key) => settingsStore.setApiKey(key)}
    />

    <GlobalSchedule
      schedule={settingsStore.globalSchedule}
      autoRun={settingsStore.schedulerAutoRun}
      launchAtStartup={settingsStore.launchAtStartup}
      onScheduleChange={(t) => settingsStore.setGlobalSchedule(t)}
      onAutoRunChange={(v) => settingsStore.setSchedulerAutoRun(v)}
      onLaunchAtStartupChange={(v) => settingsStore.setLaunchAtStartup(v)}
    />

    <div class="flex items-center gap-3">
      <button
        class="btn btn-success btn-sm"
        onclick={triggerAll}
        disabled={running}
      >
        {running ? "Running…" : "Run all now"}
      </button>
      {#if runError}
        <span class="text-xs text-red-400">{runError}</span>
      {/if}
    </div>

    <StorageProjectList globalSchedule={settingsStore.globalSchedule} />
  {/if}
</div>
