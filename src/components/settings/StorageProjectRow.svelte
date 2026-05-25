<script lang="ts">
  import { untrack } from "svelte";
  import { api, type FolderConfig, type StorageProjectItem } from "$lib/api";
  import FolderPicker from "./FolderPicker.svelte";

  interface Props {
    item: StorageProjectItem;
    folderConfigs: FolderConfig[];
    globalSchedule: string;
    names: Map<string, string>;
    onConfigsChanged: () => void;
  }

  let { item, folderConfigs, globalSchedule, names, onConfigsChanged }: Props = $props();

  let running = $state(false);
  let error = $state<string | null>(null);
  let scheduleInput = $state(untrack(() => folderConfigs[0]?.custom_schedule ?? ""));

  $effect(() => {
    scheduleInput = folderConfigs[0]?.custom_schedule ?? "";
  });

  const storageName = $derived(item.storage_id ? names.get(item.storage_id) ?? null : null);
  const projectName = $derived(item.project_id ? names.get(item.project_id) ?? null : null);

  function labelFor(prefix: string, id: string | null, name: string | null): string {
    if (!id) return `${prefix}: N/A`;
    return name ? name : `${prefix}: ${id.slice(0, 8)}…`;
  }

  async function removeFolder(id: string) {
    try {
      await api.deleteFolderConfig(id);
      onConfigsChanged();
    } catch (e) {
      error = String(e);
    }
  }

  async function saveCustomSchedule() {
    const value = scheduleInput.trim() || null;
    error = null;
    try {
      for (const fc of folderConfigs) {
        await api.upsertFolderConfig({
          storage_id: fc.storage_id,
          project_id: fc.project_id,
          folder_path: fc.folder_path,
          custom_schedule: value,
        });
      }
      onConfigsChanged();
    } catch (e) {
      error = String(e);
    }
  }

  async function runNow() {
    if (!item.storage_id || !item.project_id) return;
    running = true;
    error = null;
    try {
      await api.triggerStorageProject(item.storage_id, item.project_id);
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="border border-gray-700 rounded p-3 space-y-2">
  <!-- Header row: names/IDs + Run button -->
  <div class="flex items-start justify-between gap-2">
    <div class="flex flex-col gap-1 text-xs min-w-0">
      <span
        class="bg-gray-700 px-1.5 py-0.5 rounded text-gray-300 truncate max-w-60"
        title={item.storage_id ?? ""}
      >
        S: {labelFor("S", item.storage_id, storageName)}
      </span>
      <span
        class="bg-gray-700 px-1.5 py-0.5 rounded text-gray-300 truncate max-w-60"
        title={item.project_id ?? ""}
      >
        P: {labelFor("P", item.project_id, projectName)}
      </span>
    </div>
    {#if folderConfigs.length > 0}
      <button
        class="shrink-0 btn btn-success btn-xs"
        onclick={runNow}
        disabled={running || !item.storage_id || !item.project_id}
      >
        {running ? "…" : "Run"}
      </button>
    {/if}
  </div>

  <!-- Configured folders -->
  {#if folderConfigs.length > 0}
    <div class="space-y-1">
      {#each folderConfigs as fc (fc.id)}
        <div class="flex items-center gap-2 text-xs bg-gray-800 rounded px-2 py-1">
          <span class="flex-1 font-mono text-gray-300 truncate" title={fc.folder_path}>
            {fc.folder_path}
          </span>
          <button
            class="shrink-0 text-red-400 hover:text-red-300 px-1"
            onclick={() => removeFolder(fc.id)}
            aria-label="Remove folder"
          >
            ✕
          </button>
        </div>
      {/each}
    </div>

    <!-- Custom schedule override -->
    <div class="flex items-center gap-2">
      <span class="text-xs text-gray-500 shrink-0">Override:</span>
      <input
        type="time"
        class="bg-gray-800 border border-gray-700 rounded px-2 py-0.5 text-xs text-gray-300
               focus:outline-none focus:border-blue-500"
        placeholder={globalSchedule}
        bind:value={scheduleInput}
        onblur={saveCustomSchedule}
      />
      <span class="text-xs text-gray-600">(blank = global)</span>
    </div>
  {/if}

  <FolderPicker
    storageId={item.storage_id ?? ""}
    projectId={item.project_id ?? ""}
    onFolderAdded={onConfigsChanged}
  />

  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}
</div>
