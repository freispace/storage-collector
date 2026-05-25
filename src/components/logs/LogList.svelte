<script lang="ts">
  import { levelColor, formatDate } from "$lib/utils";
  import type { LogEntry } from "$lib/api";
  import { namesStore } from "$lib/stores/names.svelte";

  interface Props {
    entries: LogEntry[];
  }

  let { entries }: Props = $props();

  let listEl = $state<HTMLElement | null>(null);
  let autoScroll = $state(true);

  $effect(() => {
    if (autoScroll && listEl && entries.length > 0) {
      listEl.scrollTop = 0;
    }
  });

  const UUID_RE = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi;

  function resolveMessage(message: string): string {
    return message.replace(UUID_RE, (id) => {
      const name = namesStore.lookup(id);
      return name ? `${name} (${id})` : id;
    });
  }
</script>

<div
  bind:this={listEl}
  class="overflow-y-auto h-full bg-base-100 rounded-t-box shadow-md px-3 py-1"
  onscroll={(e) => {
    const el = e.currentTarget as HTMLElement;
    autoScroll = el.scrollTop < 10;
  }}
>
  {#if entries.length === 0}
    <div class="text-center text-gray-400 py-20">No log entries</div>
  {:else}
  <table class="table table-xs font-mono">
  <tbody>
    {#each entries as entry (entry.id)}
    <tr>
        <th class="shrink-0 w-36 align-top">{formatDate(entry.created_at)}</th>
        <td class="shrink-0 w-14 align-top {levelColor(entry.level)}">{entry.level}</td>
        <td>{resolveMessage(entry.message)}</td>
      </tr>
    {/each}
  </tbody>
  </table>
  {/if}
</div>
