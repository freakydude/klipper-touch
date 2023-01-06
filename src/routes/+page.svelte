<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {    
    connect(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  });
</script>

<div class="items-around grid grow grid-cols-3  grid-rows-2  gap-4 bg-neutral-800 p-4 text-center text-neutral-200">
  <a class="btn-default" href="/extruder">Extruder</a>
  <a class="btn-default" href="/move">Move</a>
  <a class="btn-default" href="/files">Files</a>
  <a class="btn-default" href="/heaterbed">Heaterbed</a>
  <a class="btn-default" href="/maschine">Maschine</a>
  <a class="btn-default" href="/progress">Progress</a>
</div>
