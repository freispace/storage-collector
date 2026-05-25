<script lang="ts">
    interface Props {
        class?: string;
    }

    let { class: classes = "" }: Props = $props();
</script>

<span class="loading loading-infinity loading-xl me-1 {classes}"></span> Loading…
