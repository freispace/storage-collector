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
  let error = $state<string | null>(null);

  $effect(() => {
    timeValue = schedule;
  });

  async function saveSchedule() {
    error = null;
    try {
      await api.setGlobalSchedule(timeValue);
      onScheduleChange(timeValue);
    } catch (e) {
      error = String(e);
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

<div>
  <label class="flex items-center gap-2 cursor-pointer">
    <input
      type="checkbox"
      class="toggle"
      checked={launchAtStartup}
      onchange={toggleLaunchAtStartup}
    />
    <span class="text-sm text-gray-300">Launch at system startup</span>
  </label>

  <label class="flex items-center gap-2 cursor-pointer mt-5">
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

  <div class="pl-12 pt-3">
    <input
      id="global-schedule-input"
      type="time"
      class="input w-28"
      bind:value={timeValue}
      onblur={saveSchedule}
      disabled={!autoRun}
    />
    {#if error}
      <p class="text-xs text-red-400">{error}</p>
    {/if}
  </div>
</div>
