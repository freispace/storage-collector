<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "$lib/api";

  interface Props {
    apiKey: string;
    onChange: (key: string) => void;
  }

  let { apiKey, onChange }: Props = $props();

  let inputValue = $state(untrack(() => apiKey));
  let saving = $state(false);
  let saved = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    inputValue = apiKey;
  });

  async function save() {
    saving = true;
    error = null;
    try {
      await api.setApiKey(inputValue);
      onChange(inputValue);
      saved = true;
      setTimeout(() => { saved = false; }, 2000);
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-2">
  <label for="api-key-input" class="block text-xs font-medium text-gray-400 uppercase tracking-wide">API Key</label>
  <div class="flex gap-2">
    <input
      id="api-key-input"
      type="password"
      class="flex-1 bg-gray-800 border border-gray-600 rounded px-3 py-1.5 text-sm text-gray-100
             focus:outline-none focus:border-blue-500"
      placeholder="freispace API key"
      bind:value={inputValue}
      onkeydown={(e) => { if (e.key === "Enter") save(); }}
    />
    <button
      class="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-500 disabled:opacity-50"
      onclick={save}
      disabled={saving}
    >
      {saving ? "Saving…" : saved ? "Saved!" : "Save"}
    </button>
  </div>
  {#if error}
    <p class="text-xs text-red-400">{error}</p>
  {/if}
</div>
