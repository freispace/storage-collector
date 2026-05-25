<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "$lib/api";

  interface Props {
    schedule: string;
    autoRun: boolean;
    launchAtStartup: boolean;
    onScheduleChange: (time: string) => void;
    onAutoRunChange: (enabled: boolean) => void;
    onLaunchAtStartupChange: (enabled: boolean) => void;
  }

  let {
    schedule,
    autoRun,
    launchAtStartup,
    onScheduleChange,
    onAutoRunChange,
    onLaunchAtStartupChange,
  }: Props = $props();

  let timeValue = $state(untrack(() => schedule));
  let saving = $state(false);
  let saved = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    timeValue = schedule;
  });

  async function saveSchedule() {
    saving = true;
    error = null;
    try {
      await api.setGlobalSchedule(timeValue);
      onScheduleChange(timeValue);
      saved = true;
      setTimeout(() => {
        saved = false;
      }, 2000);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function toggleAutoRun(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    try {
      await api.setSchedulerAutoRun(checked);
      onAutoRunChange(checked);
    } catch (err) {
      console.error(err);
    }
  }

  async function toggleLaunchAtStartup(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    try {
      await api.setLaunchAtStartup(checked);
      onLaunchAtStartupChange(checked);
    } catch (err) {
      console.error(err);
    }
  }
</script>

<div class="space-y-3">
  <div class="space-y-2">
    <label class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        class="toggle"
        checked={launchAtStartup}
        onchange={toggleLaunchAtStartup}
      />
      <span class="text-sm text-gray-300">Launch at system startup</span>
    </label>

    <label class="flex items-center gap-2 cursor-pointer">
      <input
        type="checkbox"
        class="toggle"
        checked={autoRun}
        onchange={toggleAutoRun}
      />
      <span class="text-sm text-gray-300"
        >Run automatically at scheduled time</span
      >
    </label>

    <div class="pt-4">
      <label
        for="global-schedule-input"
        class="block text-xs font-medium text-gray-400 uppercase tracking-wide"
      >
        Global Schedule (daily)
      </label>
      <p>Time that all storage projects should be updated</p>
      <div class="flex gap-2 items-center">
        <input
          id="global-schedule-input"
          type="time"
          class="bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-100
                focus:outline-none focus:border-blue-500"
          bind:value={timeValue}
        />
        <button
          class="btn btn-md btn-primary"
          onclick={saveSchedule}
          disabled={saving}
        >
          {saving ? "Saving…" : saved ? "Saved!" : "Save"}
        </button>
      </div>
    </div>
    {#if error}
      <p class="text-xs text-red-400">{error}</p>
    {/if}
  </div>
</div>
