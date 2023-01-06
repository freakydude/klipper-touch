<script lang="ts">
  let increment = 10;
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">
    <div class="flex flex-row justify-center gap-1">
      <p class="px-2">X: 150</p>
      <p class="px-2">Y: 150</p>
      <p class="px-2">Z: 50</p>
      <p class="px-2">Z-Offset: 0.02</p>
    </div>

    <div class="flex grow flex-col justify-center p-2">
      <div class="flex flex-row  ">
        <div class="flex grow flex-row justify-center">
          <div class="grid  grid-cols-3 grid-rows-3">
            <button class="col-start-1 row-start-2 btn-default">X-</button>
            <button class="col-start-3 row-start-2 btn-default">X+</button>
            <p class="col-start-2 row-start-2 self-center justify-self-center p-2">XY</p>
            <button class="col-start-2 row-start-1 btn-default">Y+</button>
            <button class="col-start-2 row-start-3 btn-default">Y-</button>
          </div>

          <div class="grid grid-cols-1 grid-rows-3 pl-6">
            <button class="col-start-1 row-start-1 btn-default">Z+</button>
            <p class="col-start-1 row-start-2 self-center justify-self-center p-2">Z</p>
            <button class="col-start-1 row-start-3 btn-default">Z-</button>
          </div>
        </div>
        <div class="grid grid-cols-1 grid-rows-3 pl-2">
          <button class="col-start-1 row-start-1 justify-self-center btn-default">O+</button>
          <p class="col-start-1 row-start-2 self-center justify-self-center p-2">Z-Offset</p>
          <button class="col-start-1 row-start-3 justify-self-center btn-default">O-</button>
        </div>
      </div>
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
