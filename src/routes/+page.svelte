<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  // onMount(async () => {
  //   connect(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  // });
</script>

<div class="flex grow flex-col items-center justify-center text-neutral-200">
  <p class="p-2 text-5xl font-bold  ">Klipper-Touch</p>
  <p class="p-2 font-bold text-red-800 ">by freakyDude</p>
</div>
