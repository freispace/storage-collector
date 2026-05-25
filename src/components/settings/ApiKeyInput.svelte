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
  <label for="api-key-input" class="block text-xs font-medium text-gray-400 uppercase tracking-wide">freispace API Key</label>
  <div class="text-gray-400">
    The API key requires the following permissions:
    <span class="badge badge-sm font-mono">Projects : View</span> <span class="badge badge-sm font-mono">Storages : Update</span>
  </div>
  <div class="flex gap-2">
    <div class="input flex-1 max-w-135">
      <svg class="h-[1em] opacity-50" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
        <g
          stroke-linejoin="round"
          stroke-linecap="round"
          stroke-width="2.5"
          fill="none"
          stroke="currentColor"
        >
          <path
            d="M2.586 17.414A2 2 0 0 0 2 18.828V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.172a2 2 0 0 0 1.414-.586l.814-.814a6.5 6.5 0 1 0-4-4z"
          ></path>
          <circle cx="16.5" cy="7.5" r=".5" fill="currentColor"></circle>
        </g>
      </svg>
      <input
        id="api-key-input"
        type="password"
        placeholder="freispace API key"
        bind:value={inputValue}
        onkeydown={(e) => { if (e.key === "Enter") save(); }}
      />
    </div>
    <button
      class="btn btn-primary"
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
