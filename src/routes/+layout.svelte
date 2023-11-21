<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import '../app.css';

  let isConnected = client.isConnected;
  let klippyState = moonraker.klippyState.state;

  $: {
    if ($isConnected === false || $klippyState !== 'ready') {
      goto('/');
    }
    if ($isConnected === true && $klippyState === 'ready') {
      goto('/printstate');
    }
  }
</script>

<div class="flex h-screen w-screen">
  <slot />
</div>
