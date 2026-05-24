<script lang="ts">
  interface Props {
    filter: string | null;
    onFilterChange: (filter: string | null) => void;
  }

  let { filter, onFilterChange }: Props = $props();

  const levels: { value: string | null; label: string }[] = [
    { value: null, label: "All" },
    { value: "info", label: "Info" },
    { value: "warning", label: "Warning" },
    { value: "error", label: "Error" },
  ];
</script>

<div class="flex gap-1">
  {#each levels as level}
    <button
      class="px-2 py-1 text-xs rounded transition-colors
        {filter === level.value
          ? 'bg-blue-600 text-white'
          : 'bg-gray-700 text-gray-300 hover:bg-gray-600'}"
      onclick={() => onFilterChange(level.value)}
    >
      {level.label}
    </button>
  {/each}
</div>
