<script lang="ts">
  import { currentScreen } from '../../store';
  import Overview from '../+page.svelte';
</script>

<div class="flex grow flex-row bg-neutral-800 text-neutral-200">
  <div class="flex  grow flex-col">
    Progress
  </div>
  <div class="flex flex-col justify-evenly pl-2">
    <button class="rounded bg-neutral-600">1</button>
    <button class="rounded bg-neutral-600">2</button>
    <button class="rounded bg-neutral-600">5</button>
    <button class="rounded bg-neutral-600">10</button>
    <button class="rounded bg-neutral-600">20</button>
    <button class="rounded bg-neutral-600">50</button>
    <button class="rounded bg-neutral-600 px-2">100</button>
  </div>
</div>
