<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect('192.168.40.6/websocket');
  });

  let increment = 10;
  let bedTemp = 65;

  function incBedTemp() {
    bedTemp += increment;
  }
  function decBedTemp() {
    if (bedTemp - increment < 0) {
      bedTemp = 0;
    } else {
      bedTemp -= increment;
    }
  }
</script>

<div class="flex grow flex-row bg-neutral-800 text-center text-neutral-200">
  <div class="flex grow flex-row justify-evenly">
    <div class="flex flex-col flex-wrap content-center justify-center gap-2">
      <p class="p-2">Heater Bed Temperature</p>
      <button class="place-self-center rounded bg-neutral-600 p-2" on:click={incBedTemp}>T+</button>
      <p class="whitespace-nowrap p-2">{bedTemp} °C</p>
      <button class=" place-self-center rounded bg-neutral-600 p-2" on:click={decBedTemp}>T-</button>
    </div>
    <div class="flex flex-col flex-wrap justify-center gap-2">
      <button class="rounded bg-neutral-600 px-2">Homing</button>
      <button class="rounded bg-neutral-600 px-2">Bed Mesh</button>
      <button class="rounded bg-neutral-600 px-2">PID</button>
      <button class="rounded bg-neutral-600 px-2">Screw</button>
      <button class="rounded bg-neutral-600 px-2">Manual</button>
    </div>
  </div>
  <div class="flex flex-col justify-evenly pl-2">
    <button
      class="rounded bg-neutral-600 px-2 {increment === 1 ? 'text-sky-600' : 'text-neutral-200'} "
      on:click={() => {
        increment = 1;
      }}>1</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 2 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 2;
      }}>2</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 5 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 5;
      }}>5</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 10 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 10;
      }}>10</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 20 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 20;
      }}>20</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 50 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 50;
      }}>50</button
    >
    <button
      class="rounded bg-neutral-600 px-2 {increment === 100 ? 'text-sky-600' : 'text-neutral-200'}"
      on:click={() => {
        increment = 100;
      }}>100</button
    >
  </div>
</div>
