<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect('192.168.40.6/websocket');
  });
</script>

<div class="flex h-full shrink justify-center bg-neutral-800 p-4">
  <div class="grid grid-cols-2 grid-rows-3 gap-2 text-center text-neutral-200 place-items-center">
    <a class="rounded bg-neutral-600 p-4" href="/extruder">Extruder</a>
    <a class="rounded bg-neutral-600 p-4" href="/move">Move</a>
    <a class="rounded bg-neutral-600 p-4" href="/files">Files</a>
    <a class="rounded bg-neutral-600 p-4" href="/heatbed">Bed</a>
    <a class="rounded bg-neutral-600 p-4" href="/maschine">Maschine</a>
    <a class="rounded bg-neutral-600 p-4" href="/progress">Progress</a>
  </div>
</div>
