<script lang="ts">
  import { untrack } from "svelte";
  import { api } from "$lib/api";

  interface Props {
    apiKey: string;
    onChange: (key: string) => void;
  }

  let { apiKey, onChange }: Props = $props();

  let inputValue = $state(untrack(() => apiKey));
  let editing = $state(false);
  let saving = $state(false);
  let saved = $state(false);
  let error = $state<string | null>(null);

  const API_KEY_PATTERN =
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\|[A-Za-z0-9]+$/i;

  function maskedKey(key: string) {
    if (!key) return "";
    if (key.length <= 5) return `${key}${"*".repeat(5)}`;
    return `${key.slice(0, 5)}${"*".repeat(key.length - 5)}`;
  }

  function startEdit() {
    inputValue = apiKey;
    error = null;
    saved = false;
    editing = true;
  }

  function cancelEdit() {
    inputValue = apiKey;
    error = null;
    saved = false;
    editing = false;
  }

  $effect(() => {
    inputValue = apiKey;
    if (!apiKey) {
      editing = true;
    }
  });

  async function save() {
    const key = inputValue.trim();
    if (!API_KEY_PATTERN.test(key)) {
      error =
        "Invalid API key format. Expected: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx|token";
      return;
    }

    saving = true;
    error = null;
    try {
      await api.setApiKey(key);
      onChange(key);
      editing = false;
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
</script>

<div class="h-26">
  <label for="api-key-input" class="fieldset-legend mb-0.5"
    >freispace API Key</label
  >
  {#if !editing && apiKey}
    <div class="flex gap-2 items-center">
      <div
        class="input flex-1 max-w-135 text-base-content/70"
        aria-label="Saved API key"
      >
        {maskedKey(apiKey)}
      </div>
      <button class="btn" onclick={startEdit}> Edit </button>
      {#if saved}
        <span class="text-xs text-green-500">Saved!</span>
      {/if}
    </div>
  {:else}
    <div class="flex gap-2">
      <div class="input flex-1 max-w-135">
        <svg
          class="h-[1em] opacity-50"
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
        >
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
          onkeydown={(e) => {
            if (e.key === "Enter") save();
          }}
        />
      </div>
      <button class="btn btn-primary" onclick={save} disabled={saving}>
        {saving ? "Saving…" : "Save"}
      </button>
      {#if apiKey}
        <button class="btn" onclick={cancelEdit} disabled={saving}>
          Cancel
        </button>
      {/if}
    </div>
    <div class="text-gray-400 mt-2">
      The API key requires the following permissions:
      <span class="badge badge-sm font-mono">Projects : Update</span>
      <span class="badge badge-sm font-mono">Storages : Update</span>
    </div>
  {/if}
  {#if error}
    <p class="text-xs text-red-400 mt-1">{error}</p>
  {/if}
</div>
