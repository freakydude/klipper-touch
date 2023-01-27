<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<div class="flex grow flex-col flex-wrap items-center justify-center bg-neutral-800">
  <div class="flex grow flex-row flex-wrap items-center justify-center gap-2">
    <button class="btn-touch w-40" on:click={() => goto('/toolhead')}>Toolhead</button>
    <button class="btn-touch w-40" on:click={() => goto('/parameter')}>Parameter</button>
  </div>
  <p class="px-2 pt-24 text-5xl font-bold  ">Klipper-Touch</p>
  <p class="p-2 text-2xl font-bold text-red-800">by freakyDude</p>
</div>
