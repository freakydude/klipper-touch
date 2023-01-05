<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect('192.168.40.6/websocket');
  });
</script>

<div class="flex grow flex-row flex-wrap content-evenly justify-evenly gap-4 bg-neutral-800 text-center text-neutral-200">
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/extruder">Extruder</a>
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/move">Move</a>
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/files">Files</a>
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/heaterbed">Bed</a>
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/maschine">Maschine</a>
  <a class="basis-1/3 rounded bg-neutral-600 py-2" href="/progress">Progress</a>
</div>
