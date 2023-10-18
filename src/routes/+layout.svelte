<script lang="ts">
import { moonraker } from '$lib/base.svelte';
import { onMount } from 'svelte';
import '../app.css';

async function reconnectToMoonraker() {
  await moonraker.disconnect();
  await moonraker.connect();
}

onMount(async () => {
  await reconnectToMoonraker();
});
</script>

<div class="flex h-screen w-screen">
  <slot />
</div>
