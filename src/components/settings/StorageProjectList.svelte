<script lang="ts">
  import { onMount } from "svelte";
  import {
    api,
    type FolderConfig,
    type StorageProjectItem,
    type StorageProjectSetting,
  } from "$lib/api";
  import { namesStore } from "$lib/stores/names.svelte";
  import StorageProjectRow from "./StorageProjectRow.svelte";
  import LoadingText from "../general/loadingText.svelte";

  interface Props {
    globalSchedule: string;
  }

  let { globalSchedule }: Props = $props();

  let allItems = $state<StorageProjectItem[]>([]);
  let folderConfigs = $state<FolderConfig[]>([]);
  let projectSettings = $state<StorageProjectSetting[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let selectedItemId = $state("");
  let addingProject = $state(false);

  // Keys of all locally configured (storage_id, project_id) pairs
  const activeKeys = $derived.by(() => {
    const keys = new Set<string>();
    for (const fc of folderConfigs)
      keys.add(`${fc.storage_id}-${fc.project_id}`);
    for (const s of projectSettings)
      keys.add(`${s.storage_id}-${s.project_id}`);
    return keys;
  });

  // Items from the API that are configured, plus synthetic stubs for any local
  // config whose API item wasn't returned (e.g. after an API error)
  const activeItems = $derived.by(() => {
    const fromApi = allItems.filter(
      (item) =>
        item.storage_id &&
        item.project_id &&
        activeKeys.has(`${item.storage_id}-${item.project_id}`),
    );
    const seen = new Set(fromApi.map((i) => `${i.storage_id}-${i.project_id}`));

    for (const fc of folderConfigs) {
      const key = `${fc.storage_id}-${fc.project_id}`;
      if (!seen.has(key)) {
        seen.add(key);
        fromApi.push({
          id: key,
          storage_id: fc.storage_id,
          project_id: fc.project_id,
          storage_size_estimated: null,
          storage_size_current: null,
        });
      }
    }
    for (const s of projectSettings) {
      const key = `${s.storage_id}-${s.project_id}`;
      if (!seen.has(key)) {
        seen.add(key);
        fromApi.push({
          id: key,
          storage_id: s.storage_id,
          project_id: s.project_id,
          storage_size_estimated: null,
          storage_size_current: null,
        });
      }
    }
    return fromApi;
  });

  const availableItems = $derived(
    allItems.filter(
      (item) =>
        item.storage_id &&
        item.project_id &&
        !activeKeys.has(`${item.storage_id}-${item.project_id}`),
    ),
  );

  function itemLabel(item: StorageProjectItem): string {
    const p = item.project_id
      ? (namesStore.names.get(item.project_id) ??
        `${item.project_id.slice(0, 8)}…`)
      : "N/A";
    const s = item.storage_id
      ? (namesStore.names.get(item.storage_id) ??
        `${item.storage_id.slice(0, 8)}…`)
      : "N/A";
    return `${p} – ${s}`;
  }

  async function loadAll() {
    loading = true;
    error = null;

    try {
      const [configs, settings] = await Promise.all([
        api.listFolderConfigs(),
        api.listStorageProjectSettings(),
      ]);
      folderConfigs = configs as FolderConfig[];
      projectSettings = settings as StorageProjectSetting[];
    } catch (e) {
      error = String(e);
      loading = false;
      return;
    }

    try {
      let page = 1;
      const collected: StorageProjectItem[] = [];
      while (true) {
        const result = await api.fetchStorageProjectsPage(page);
        collected.push(...result.data);
        if (!result.pagination_links?.next) break;
        page++;
      }
      allItems = collected;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }

    namesStore.load();
    api
      .syncEntityNames()
      .catch(() => {})
      .then(() => namesStore.load());
  }

  async function addProject() {
    if (!selectedItemId) return;
    const item = allItems.find((i) => i.id === selectedItemId);
    if (!item?.storage_id || !item?.project_id) return;
    addingProject = true;
    try {
      await api.setStorageProjectEnabled(
        item.storage_id,
        item.project_id,
        true,
      );
      projectSettings = await api.listStorageProjectSettings();
      selectedItemId = "";
    } catch (e) {
      error = String(e);
    } finally {
      addingProject = false;
    }
  }

  async function refreshConfigs() {
    folderConfigs = await api.listFolderConfigs();
  }

  async function refreshSettings() {
    projectSettings = await api.listStorageProjectSettings();
  }

  async function removeProject(item: StorageProjectItem) {
    if (!item.storage_id || !item.project_id) return;
    try {
      await api.removeStorageProject(item.storage_id, item.project_id);
      [folderConfigs, projectSettings] = await Promise.all([
        api.listFolderConfigs(),
        api.listStorageProjectSettings(),
      ]);
    } catch (e) {
      error = String(e);
    }
  }

  function getConfigsForItem(item: StorageProjectItem): FolderConfig[] {
    return folderConfigs.filter(
      (fc) =>
        fc.storage_id === item.storage_id && fc.project_id === item.project_id,
    );
  }

  function isItemEnabled(item: StorageProjectItem): boolean {
    if (!item.storage_id || !item.project_id) return true;
    const setting = projectSettings.find(
      (s) =>
        s.storage_id === item.storage_id && s.project_id === item.project_id,
    );
    return setting?.enabled ?? true;
  }

  onMount(() => {
    loadAll();
  });
</script>

<div class="h-full flex flex-col">
  {#if loading}
    <div class="text-center py-8"><LoadingText /></div>
  {:else}
    <div class="p-6 space-y-2">
      {#if error}
        <div class="text-sm text-red-400 bg-red-900/20 rounded px-2 py-1">
          {error}
        </div>
      {/if}

      <div class="flex items-center gap-2">
        <button
          class="btn btn-ghost"
          onclick={() => loadAll()}
          aria-label="Refresh list"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-4"
            viewBox="0 0 24 24"
            ><path
              fill="currentColor"
              d="M12 20q-3.35 0-5.675-2.325T4 12t2.325-5.675T12 4q1.725 0 3.3.712T18 6.75V4h2v7h-7V9h4.2q-.8-1.4-2.187-2.2T12 6Q9.5 6 7.75 7.75T6 12t1.75 4.25T12 18q1.925 0 3.475-1.1T17.65 14h2.1q-.7 2.65-2.85 4.325T12 20"
            /></svg
          >
        </button>

        <!-- Add project dropdown -->
        {#if availableItems.length > 0}
          <select class="select flex-1" bind:value={selectedItemId}>
            <option value="">Select a storage project to add…</option>
            {#each availableItems as item (item.id)}
              <option value={item.id}>{itemLabel(item)}</option>
            {/each}
          </select>
          <button
            class="btn btn-primary"
            onclick={addProject}
            disabled={!selectedItemId || addingProject}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              class="size-4"
              viewBox="0 0 16 16"
              fill="currentColor"
            >
              <path
                d="M8.75 3.75a.75.75 0 0 0-1.5 0v3.5h-3.5a.75.75 0 0 0 0 1.5h3.5v3.5a.75.75 0 0 0 1.5 0v-3.5h3.5a.75.75 0 0 0 0-1.5h-3.5v-3.5Z"
              />
            </svg>
            <span>Add</span>
          </button>
        {/if}
      </div>
    </div>

    <div class="divider m-0"></div>

    <!-- Active projects list -->
    {#if activeItems.length === 0}
      {#if !error}
        <div class="text-center text-gray-400 py-4 text-sm">
          {allItems.length === 0
            ? "No storage projects found. Make sure your API key is configured."
            : "No storage projects added yet. Select one from the dropdown above."}
        </div>
      {/if}
    {:else}
      <div class="h-full overflow-y-scroll space-y-4 p-6">
        {#each activeItems as item (item.id)}
          <StorageProjectRow
            {item}
            folderConfigs={getConfigsForItem(item)}
            {globalSchedule}
            names={namesStore.names}
            onConfigsChanged={refreshConfigs}
            enabled={isItemEnabled(item)}
            onEnabledChanged={refreshSettings}
            onRemove={() => removeProject(item)}
          />
        {/each}
      </div>
    {/if}
  {/if}
</div>
