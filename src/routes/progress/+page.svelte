<script lang="ts">
  let increment = 10;
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">Progress</div>
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
