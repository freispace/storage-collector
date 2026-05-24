<script lang="ts">
  import { api } from "$lib/api";

  interface Props {
    storageId: string;
    projectId: string;
    onFolderAdded: () => void;
  }

  let { storageId, projectId, onFolderAdded }: Props = $props();

  let picking = $state(false);
  let error = $state<string | null>(null);

  async function pickFolder() {
    picking = true;
    error = null;
    try {
      const path = await api.pickFolder();
      if (path) {
        await api.upsertFolderConfig({
          storage_id: storageId,
          project_id: projectId,
          folder_path: path,
          custom_schedule: null,
        });
        onFolderAdded();
      }
    } catch (e) {
      error = String(e);
    } finally {
      picking = false;
    }
  }
</script>

<div>
  <button
    class="flex items-center gap-1 px-2 py-1 text-xs bg-gray-700 text-gray-300 rounded
           hover:bg-gray-600 disabled:opacity-50"
    onclick={pickFolder}
    disabled={picking || !storageId || !projectId}
  >
    + Add folder
  </button>
  {#if error}
    <p class="text-xs text-red-400 mt-1">{error}</p>
  {/if}
</div>
