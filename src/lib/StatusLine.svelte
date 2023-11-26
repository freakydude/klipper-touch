<script lang="ts">
  import { moonraker } from './base.svelte';

  let displayStatusMessage = moonraker.displayStatus.Message;

  let statusLine: string;

  $: {
    statusLine = $displayStatusMessage;
  }
</script>

<div class="flex h-6 w-full flex-row items-start justify-center">
  <p class="overflow-clip text-sm text-neutral-50">{statusLine}</p>
</div>
