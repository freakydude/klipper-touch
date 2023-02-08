<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<div class="flex grow flex-col flex-wrap place-content-around items-center bg-neutral-800">
  <div class="flex flex-col flex-wrap items-center justify-center gap-2">
    <p class="px-2 text-5xl font-bold text-white">Klipper Touch</p>
    <p class="p-2 text-2xl font-bold text-red-700">by freakyDude</p>
  </div>
  <div class="flex flex-row flex-wrap items-center justify-center gap-2 ">
    <button class="btn-touch" on:click={() => goto('/toolhead')}>Toolhead</button>
    <button class="btn-touch" on:click={() => goto('/printerstatus')}>Printerstatus</button>
    <button class="btn-touch" on:click={() => goto('/parameter')}>Parameter</button>
  </div>
</div>
