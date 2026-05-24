<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "$lib/api";

  interface Props {
    schedule: string;
    autoRun: boolean;
    onScheduleChange: (time: string) => void;
    onAutoRunChange: (enabled: boolean) => void;
  }

  let { schedule, autoRun, onScheduleChange, onAutoRunChange }: Props = $props();

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
      setTimeout(() => { saved = false; }, 2000);
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
</script>

<div class="space-y-3">
  <div class="space-y-2">
    <label for="global-schedule-input" class="block text-xs font-medium text-gray-400 uppercase tracking-wide">
      Global Schedule (daily)
    </label>
    <div class="flex gap-2 items-center">
      <input
        id="global-schedule-input"
        type="time"
        class="bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-100
               focus:outline-none focus:border-blue-500"
        bind:value={timeValue}
      />
      <button
        class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-500 disabled:opacity-50"
        onclick={saveSchedule}
        disabled={saving}
      >
        {saving ? "Saving…" : saved ? "Saved!" : "Save"}
      </button>
    </div>
    {#if error}
      <p class="text-xs text-red-400">{error}</p>
    {/if}
  </div>

  <label class="flex items-center gap-2 cursor-pointer">
    <input
      type="checkbox"
      class="w-4 h-4 rounded bg-gray-700 border-gray-600"
      checked={autoRun}
      onchange={toggleAutoRun}
    />
    <span class="text-sm text-gray-300">Run automatically at scheduled time</span>
  </label>
</div>
