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

<div role="tablist" class="tabs tabs-box">
  {#each levels as level}
    <button
      class="tab
        {filter === level.value
          ? 'tab-active'
          : ''}"
      onclick={() => onFilterChange(level.value)}
    >
      {level.label}
    </button>
  {/each}
</div>
