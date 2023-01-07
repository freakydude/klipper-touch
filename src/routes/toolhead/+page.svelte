<script lang="ts">
  let distance = 10;

  function incExtrudeRetract() {
    // extrude selected increment
  }
  function decExtrudeRetract() {
    // retract selected increment
  }
</script>

<div class="flex grow flex-row items-center justify-evenly gap-x-2 bg-neutral-900 text-neutral-200">
  <div class="grid grid-cols-4 grid-rows-4 items-stretch gap-2 rounded  bg-neutral-800 p-3 text-center">
    <div class="col-span-2 col-start-1 row-start-1 flex flex-col self-start py-2">
      <p>Move X, Y, Z</p>
      <p>Distance: {distance}</p>
    </div>
    <button class="btn-default x col-span-2 col-start-3 row-start-1 self-start">Home</button>

    <div class="btn-default col-start-1 row-start-2 flex flex-col items-center justify-center">
      <p>X-</p>
      <p>Y+</p>
    </div>
    <div class="btn-default col-start-2 row-start-2 flex aspect-square items-center justify-center">Y+</div>
    <div class="btn-default col-start-3 row-start-2 flex aspect-square flex-col items-center justify-center">
      <p>X+</p>
      <p>Y+</p>
    </div>

    <div class="btn-default col-start-1 row-start-3 flex aspect-square items-center justify-center">X-</div>
    <div class="col-start-2 row-start-3 flex aspect-square flex-col items-center justify-center">
      <p>X: 134</p>
      <p>Y: 256</p>
    </div>
    <div class="btn-default col-start-3 row-start-3 flex aspect-square items-center justify-center">X+</div>

    <div class="btn-default col-start-1 row-start-4 flex aspect-square flex-col items-center justify-center">
      <p>X-</p>
      <p>Y-</p>
    </div>
    <div class="btn-default col-start-2 row-start-4 flex aspect-square items-center justify-center">Y-</div>
    <div class="btn-default col-start-3 row-start-4 flex aspect-square flex-col items-center justify-center">
      <p>X+</p>
      <p>Y-</p>
    </div>

    <div class="btn-default col-start-4 row-start-2 flex aspect-square items-center justify-center">Z+</div>
    <div class="col-start-4 row-start-3  flex flex-col  items-center justify-center">
      <div>Z: 111</div>
    </div>
    <div class="btn-default col-start-4 row-start-4 flex aspect-square items-center justify-center">Z-</div>
  </div>

  <div class="flex flex-col gap-1 rounded bg-neutral-800 p-3">
    <p class="self-start py-2">Distance</p>
    <button
      class={distance === 1 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 1;
      }}>1</button
    >
    <button
      class={distance === 2 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 2;
      }}>2</button
    >
    <button
      class={distance === 5 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 5;
      }}>5</button
    >
    <button
      class={distance === 10 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 10;
      }}>10</button
    >
    <button
      class={distance === 20 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 20;
      }}>20</button
    >
    <button
      class={distance === 50 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 50;
      }}>50</button
    >
    <button
      class={distance === 100 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 100;
      }}>100</button
    >
  </div>

  <div class="grid grid-cols-1 grid-rows-4 items-start gap-2 rounded bg-neutral-800 p-3 text-center">
    <p class="col-start-1 row-start-1 py-2">Filament</p>
    <div class="btn-default col-start-1 row-start-2 flex items-center  justify-center">Retract</div>
    <div class="col-start-1 row-start-3 flex flex-col items-center justify-center ">
      <p>Distance: {distance}</p>
      <p>Speed: 5</p>
    </div>
    <div class="btn-default col-start-1 row-start-4 flex items-center justify-center">Extrude</div>
  </div>
</div>
