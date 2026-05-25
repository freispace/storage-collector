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

<div class="join">
  {#each levels as level}
    <button
      class="join-item btn
        {filter === level.value
          ? 'btn-primary'
          : ''}"
      onclick={() => onFilterChange(level.value)}
    >
      {level.label}
    </button>
  {/each}
</div>