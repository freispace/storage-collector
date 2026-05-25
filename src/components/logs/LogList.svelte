<script lang="ts">
  import { levelColor, formatDate } from "$lib/utils";
  import type { LogEntry } from "$lib/api";

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
</script>

<div
  bind:this={listEl}
  class="overflow-y-auto h-full font-mono text-xs bg-base-100 rounded-t-box shadow-md mx-3"
  onscroll={(e) => {
    const el = e.currentTarget as HTMLElement;
    autoScroll = el.scrollTop < 10;
  }}
>
  {#if entries.length === 0}
    <div class="text-center text-gray-500 py-8">No log entries</div>
  {:else}
    {#each entries as entry (entry.id)}
      <div class="flex gap-2 px-3 py-1 border-b border-gray-800 hover:bg-gray-800/50">
        <span class="text-gray-500 shrink-0 w-36">{formatDate(entry.created_at)}</span>
        <span class="shrink-0 w-14 {levelColor(entry.level)}">{entry.level}</span>
        <span class="text-gray-200 break-all">{entry.message}</span>
      </div>
    {/each}
  {/if}
</div>
