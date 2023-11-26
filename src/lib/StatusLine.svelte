<script lang="ts">
  import { moonraker } from './base.svelte';

  let displayStatusMessage = moonraker.displayStatus.Message;
</script>

<div class="flex h-6 w-full flex-row items-start justify-center">
  <p class="overflow-clip text-sm text-neutral-50">{$displayStatusMessage}</p>
</div>
