<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<div class="flex flex-grow flex-row flex-wrap place-content-center items-center gap-2 bg-neutral-800">
  <button class="btn-touch" on:click={() => goto('/toolhead')}>Toolhead</button>
  <button class="btn-touch" on:click={() => goto('/files')}>Files</button>
  <button class="btn-touch" on:click={() => goto('/printerstatus')}>Printerstatus</button>
  <button class="btn-touch" on:click={() => goto('/parameter')}>Parameter</button>
</div>
