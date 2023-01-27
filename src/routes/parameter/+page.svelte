<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<div class="flex grow flex-col flex-wrap place-content-around items-center bg-neutral-800">
  <p class="label text-4xl">Parameters</p>
  <div class="flex flex-row flex-wrap items-center gap-2">
    <button class="btn-touch h-32 w-64" on:click={() => goto('/parameter/babysteps')}>Babysteps</button>
    <button class="btn-touch h-32 w-64" on:click={() => goto('/parameter/temperatures')}>Temperatures</button>
  </div>
</div>
