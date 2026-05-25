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
    enabled: boolean;
    onEnabledChanged: () => void;
    onRemove: () => void;
  }

  let {
    item,
    folderConfigs,
    globalSchedule,
    names,
    onConfigsChanged,
    enabled,
    onEnabledChanged,
    onRemove,
  }: Props = $props();

  let running = $state(false);
  let error = $state<string | null>(null);
  let confirming = $state(false);
  let scheduleInput = $state(
    untrack(() => folderConfigs[0]?.custom_schedule ?? ""),
  );

  $effect(() => {
    scheduleInput = folderConfigs[0]?.custom_schedule ?? "";
  });

  const storageName = $derived(
    item.storage_id ? (names.get(item.storage_id) ?? null) : null,
  );
  const projectName = $derived(
    item.project_id ? (names.get(item.project_id) ?? null) : null,
  );

  function labelFor(
    prefix: string,
    id: string | null,
    name: string | null,
  ): string {
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

  async function toggleEnabled() {
    if (!item.storage_id || !item.project_id) return;
    error = null;
    try {
      await api.setStorageProjectEnabled(
        item.storage_id,
        item.project_id,
        !enabled,
      );
      onEnabledChanged();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div
  class="card card-border bg-base-100 px-4 py-3 space-y-3"
  class:opacity-60={!enabled}
>
  <!-- Header row: names/IDs + controls -->
  <div class="flex items-start justify-between gap-2">
    <h1 class="font-medium flex items-center gap-2">
      <input
        type="checkbox"
        class="toggle toggle-xs"
        checked={enabled}
        onchange={toggleEnabled}
      />
      {labelFor("P", item.project_id, projectName)} &ndash; {labelFor(
        "S",
        item.storage_id,
        storageName,
      )}
    </h1>
    <div class="flex items-center gap-2 shrink-0">
      {#if confirming}
        <span class="text-xs text-gray-400">Remove this project?</span>
        <button class="btn btn-error btn-xs" onclick={onRemove}>Remove</button>
        <button
          class="btn btn-ghost btn-xs"
          onclick={() => (confirming = false)}>Cancel</button
        >
      {:else}
        {#if folderConfigs.length > 0}
          <button
            class="btn btn-success btn-xs"
            onclick={runNow}
            disabled={running || !item.storage_id || !item.project_id}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 16 16"
              fill="currentColor"
              class="size-4"
            >
              <path
                d="M3 3.732a1.5 1.5 0 0 1 2.305-1.265l6.706 4.267a1.5 1.5 0 0 1 0 2.531l-6.706 4.268A1.5 1.5 0 0 1 3 12.267V3.732Z"
              />
            </svg>
            <span>{running ? "…" : "Run"}</span>
          </button>
        {/if}
        <button
          class="btn btn-ghost btn-xs text-gray-500 hover:text-red-400"
          onclick={() => (confirming = true)}
          title="Remove project"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 16 16"
            fill="currentColor"
            class="size-4"
          >
            <path
              fill-rule="evenodd"
              d="M5 3.25V4H2.75a.75.75 0 0 0 0 1.5h.3l.815 8.15A1.5 1.5 0 0 0 5.357 15h5.285a1.5 1.5 0 0 0 1.493-1.35l.815-8.15h.3a.75.75 0 0 0 0-1.5H11v-.75A2.25 2.25 0 0 0 8.75 1h-1.5A2.25 2.25 0 0 0 5 3.25Zm2.25-.75a.75.75 0 0 0-.75.75V4h3v-.75a.75.75 0 0 0-.75-.75h-1.5ZM6.05 6a.75.75 0 0 1 .787.713l.275 5.5a.75.75 0 0 1-1.498.075l-.275-5.5A.75.75 0 0 1 6.05 6Zm3.9 0a.75.75 0 0 1 .712.787l-.275 5.5a.75.75 0 0 1-1.498-.075l.275-5.5a.75.75 0 0 1 .786-.712Z"
              clip-rule="evenodd"
            />
          </svg>
        </button>
      {/if}
    </div>
  </div>

  <!-- Configured folders -->
  {#if folderConfigs.length > 0}
    <div class="space-y-1">
      {#each folderConfigs as fc (fc.id)}
        <div
          class="flex items-center gap-2 text-xs bg-gray-800 rounded px-2 py-1"
        >
          <span
            class="flex-1 font-mono text-gray-300 truncate"
            title={fc.folder_path}
          >
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
  {/if}

  <FolderPicker
    storageId={item.storage_id ?? ""}
    projectId={item.project_id ?? ""}
    onFolderAdded={onConfigsChanged}
  />

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

  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}
</div>
