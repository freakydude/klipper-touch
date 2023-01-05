<script lang="ts">
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">
    <div class="flex flex-row justify-center gap-1">
      <p class="px-2">X: 150</p>
      <p class="px-2">Y: 150</p>
      <p class="px-2">Z: 50</p>
      <p class="px-2">Z-Offset: 0.02</p>
    </div>

    <div class="flex grow flex-col p-2 ">
      <div class="flex grow flex-row ">
        <div class="flex grow flex-row justify-center">
          <div class="grid  grid-cols-3 grid-rows-3">
            <button class="col-start-1 row-start-2 rounded bg-neutral-600 p-2">X-</button>
            <button class="col-start-3 row-start-2 rounded bg-neutral-600 p-2">X+</button>
            <p class="col-start-2 row-start-2 self-center justify-self-center p-2">XY</p>
            <button class="col-start-2 row-start-1 rounded bg-neutral-600 p-2">Y+</button>
            <button class="col-start-2 row-start-3 rounded bg-neutral-600 p-2">Y-</button>
          </div>

          <div class="grid grid-cols-1 grid-rows-3 pl-2">
            <button class="col-start-1 row-start-1 rounded bg-neutral-600 p-2">Z+</button>
            <p class="col-start-1 row-start-2  p-2">Z</p>
            <button class="col-start-1 row-start-3 rounded bg-neutral-600 p-2">Z-</button>
          </div>
        </div>
        <div class="grid grid-cols-1 grid-rows-3">
          <button class="col-start-1 row-start-1 justify-self-center rounded bg-neutral-600 p-2">O+</button>
          <p class="col-start-1 row-start-2 p-2">Z-Offset</p>
          <button class="col-start-1 row-start-3 justify-self-center rounded bg-neutral-600 p-2">O-</button>
        </div>
      </div>
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
