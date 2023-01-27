<script lang="ts">
  let increment = 10;
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">Progress</div>
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
