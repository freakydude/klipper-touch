<script lang="ts">

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
      <button class="place-self-center btn-default" on:click={incBedTemp}>T+</button>
      <p class="whitespace-nowrap p-2">{bedTemp} °C</p>
      <button class=" place-self-center btn-default" on:click={decBedTemp}>T-</button>
    </div>
    <div class="flex flex-col flex-wrap justify-center gap-2">
      <button class="btn-default">Homing</button>
      <button class="btn-default">Bed Mesh</button>
      <button class="btn-default">PID</button>
      <button class="btn-default">Screw</button>
      <button class="btn-default">Manual</button>
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
