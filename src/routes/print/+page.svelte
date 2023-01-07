<script lang="ts">
  import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  // onMount(async () => {
  //   connect(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  // });

  let increment = 10;
  let extTemp = 210;
  let fanSpeed = 75;
  let flowRate = 100;

  function incExtTemp() {
    extTemp += increment;
  }
  function decExtTemp() {
    if (extTemp - increment < 0) {
      extTemp = 0;
    } else {
      extTemp -= increment;
    }
  }

  function incFanSpeed() {
    fanSpeed += increment;
  }
  function decFanSpeed() {
    if (fanSpeed - increment < 0) {
      fanSpeed = 0;
    } else {
      fanSpeed -= increment;
    }
  }

  function incFlowRate() {
    flowRate += increment;
  }
  function decFlowRate() {
    if (flowRate - increment < 0) {
      flowRate = 0;
    } else {
      flowRate -= increment;
    }
  }
</script>

<div class="flex grow flex-row bg-neutral-800 text-center text-neutral-200">
  <div class="flex grow flex-row justify-evenly">
    <div class="flex flex-col flex-wrap content-center justify-center">
      <p class="p-2">Extruder Temp</p>
      <button class="btn-default place-self-center" on:click={incExtTemp}>T+</button>
      <p class="whitespace-nowrap p-2">{extTemp} °C</p>
      <button class="btn-default place-self-center" on:click={decExtTemp}>T-</button>
    </div>

    <div class="flex flex-col flex-wrap content-center justify-center">
      <p class="p-2">Fan Speed</p>
      <button class="btn-default place-self-center" on:click={incFanSpeed}>S+</button>
      <p class="whitespace-nowrap p-2">{fanSpeed} %</p>
      <button class="btn-default place-self-center" on:click={decFanSpeed}>S-</button>
    </div>
    <div class="flex flex-col flex-wrap content-center justify-center">
      <p class="p-2">Flow Rate</p>
      <button class="btn-default place-self-center" on:click={incFlowRate}>F+</button>
      <p class="whitespace-nowrap p-2">{flowRate} %</p>
      <button class="btn-default place-self-center" on:click={decFlowRate}>F-</button>
    </div>
    <div class="flex flex-col flex-wrap content-center justify-center">
      <button class="btn-default place-self-center">O+</button>
      <p class="whitespace-nowrap p-2">Z-Offset</p>
      <button class="btn-default place-self-center">O-</button>
    </div>

    <div class="ml-2 flex flex-col  flex-wrap justify-center gap-2">
      <button class="btn-default">Homing</button>
      <button class="btn-default">Bed Mesh</button>
      <button class="btn-default">PID</button>
    </div>
  </div>
  <div class="ml-2 flex flex-col justify-evenly">
    <button
      class="py-1 {increment === 1 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 1;
      }}>1</button
    >
    <button
      class="py-1 {increment === 2 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 2;
      }}>2</button
    >
    <button
      class="py-1 {increment === 5 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 5;
      }}>5</button
    >
    <button
      class="py-1 {increment === 10 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 10;
      }}>10</button
    >
    <button
      class="py-1 {increment === 20 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 20;
      }}>20</button
    >
    <button
      class="py-1 {increment === 50 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 50;
      }}>50</button
    >
    <button
      class="py-1 {increment === 100 ? 'btn-primary' : 'btn-default'}"
      on:click={() => {
        increment = 100;
      }}>100</button
    >
  </div>
</div>
