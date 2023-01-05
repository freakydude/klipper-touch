<script lang="ts">
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">
    <div class="flex flex-row justify-center gap-1">
      <p>X: 123</p>
      <p>Y: 123</p>
      <p>Z: 123</p>
      <p>S: 100</p>
      <p>Z-Off: 0.02</p>
    </div>

    <div class="flex grow flex-col p-2 ">
      <div class="flex grow  flex-row content-evenly justify-evenly">
        <div class="grid basis-1/2 grid-cols-3 grid-rows-3">
          <button class="col-start-1 row-start-2 rounded bg-neutral-600">X-</button>
          <button class="col-start-3 row-start-2 rounded bg-neutral-600">X+</button>
          <button class="col-start-2 row-start-1 rounded bg-neutral-600">Y+</button>
          <button class="col-start-2 row-start-3 rounded bg-neutral-600">Y-</button>
        </div>

        <div class="grid basis-1/6 grid-cols-1 grid-rows-3">
          <button class="col-start-1 row-start-1 rounded bg-neutral-600">Z+</button>
          <button class="col-start-1 row-start-3 rounded bg-neutral-600">Z-</button>
        </div>
      </div>
    </div>
  </div>
  <div class="flex basis-1/6 flex-col justify-evenly px-2">
    <button class="rounded bg-neutral-600">1</button>
    <button class="rounded bg-neutral-600">2</button>
    <button class="rounded bg-neutral-600">5</button>
    <button class="rounded bg-neutral-600">10</button>
    <button class="rounded bg-neutral-600">20</button>
    <button class="rounded bg-neutral-600">50</button>
    <button class="rounded bg-neutral-600">100</button>
  </div>
</div>
