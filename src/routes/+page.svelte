<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect('192.168.40.6/websocket');
  });
</script>

<div class="items-around grid grow grid-cols-3  grid-rows-2  gap-4 bg-neutral-800 p-4 text-center text-neutral-200">
  <a class="rounded bg-neutral-600 p-2" href="/extruder">Extruder</a>
  <a class="rounded bg-neutral-600 p-2" href="/move">Move</a>
  <a class="rounded bg-neutral-600 p-2" href="/files">Files</a>
  <a class="rounded bg-neutral-600 p-2" href="/heaterbed">Heaterbed</a>
  <a class="rounded bg-neutral-600 p-2" href="/maschine">Maschine</a>
  <a class="rounded bg-neutral-600 p-2" href="/progress">Progress</a>
</div>
