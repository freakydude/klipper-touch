<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { moonraker } from './base.svelte';
  import { definePrinterObjects } from '$lib/moonraker/types/IPrinterObjects';

  const subscription = definePrinterObjects({
    display_status: ['message']
  });

  onMount(async () => {
    await moonraker.subscribe(subscription);
  });

  onDestroy(async () => {
    await moonraker.unsubscribe(subscription);
  });

  let displayStatusMessage = moonraker.display_status.message;
</script>

<div class="flex h-6 w-full flex-row items-start justify-center">
  <p class="overflow-clip text-sm text-neutral-50">{$displayStatusMessage}</p>
</div>
