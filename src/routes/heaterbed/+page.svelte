<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect('192.168.40.6/websocket');
  });
</script>

<div class="flex grow flex-row bg-neutral-800 text-center text-neutral-200">
  <div class="flex grow flex-row justify-evenly">
    <div class="flex  flex-col flex-wrap content-center justify-center gap-2">
      <p class="p-2">Heater Bed Temperature</p>
      <button class="place-self-center rounded bg-neutral-600 p-2">T+</button>
      <p class="p-2">210 °C</p>
      <button class=" place-self-center rounded bg-neutral-600 p-2">T-</button>
    </div>
    <div class="flex  flex-col flex-wrap  justify-center gap-2">
      <button class="rounded bg-neutral-600 px-2">Homing</button>
      <button class="rounded bg-neutral-600 px-2">Bed Mesh</button>
      <button class="rounded bg-neutral-600 px-2">PID</button>
      <button class="rounded bg-neutral-600 px-2">Screw</button>
      <button class="rounded bg-neutral-600 px-2">Manual</button>
    </div>
  </div>
  <div class="flex flex-col justify-evenly pl-2">
    <button class="rounded bg-neutral-600 px-2">1</button>
    <button class="rounded bg-neutral-600 px-2">2</button>
    <button class="rounded bg-neutral-600 px-2">5</button>
    <button class="rounded bg-neutral-600 px-2">10</button>
    <button class="rounded bg-neutral-600 px-2">20</button>
    <button class="rounded bg-neutral-600 px-2">50</button>
    <button class="rounded bg-neutral-600 px-2">100</button>
  </div>
</div>
