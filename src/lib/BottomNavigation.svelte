<script lang="ts">
  import { page } from '$app/stores';
  import { commands, moonraker, values } from './base.svelte';

  let printStatsState = moonraker.printStats.State;
  let clockFormatter = values.clockFormatter;
  let clock = values.clock;

  let confirmKill = false;
</script>

<div class="flex h-11 w-full flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
  <a
    href="/printstate"
    class="flex w-16 items-center justify-center rounded-b-lg
    {$page.url.pathname === '/printstate' ? 'bg-neutral-500' : 'bg-neutral-600'} 
      px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
    State
  </a>
  {#if $printStatsState !== 'printing'}
    <a
      class="flex w-16 items-center justify-center rounded-b-lg
      {$page.url.pathname === '/move' ? 'bg-neutral-500' : 'bg-neutral-600'} 
       px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      href="/move">
      Move
    </a>
  {/if}
  <a
    href="/temperature"
    class="flex w-16 items-center justify-center rounded-b-lg
    {$page.url.pathname === '/temperature' ? 'bg-neutral-500' : 'bg-neutral-600'} 
    px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
    Temp
  </a>
  <a
    href="/babysteps"
    class="flex w-16 items-center justify-center rounded-b-lg
    {$page.url.pathname === '/babysteps' ? 'bg-neutral-500' : 'bg-neutral-600'} 
    px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
    Baby
  </a>
  {#if $printStatsState !== 'printing'}
    <a
      href="/extrusion"
      class="flex w-16 items-center justify-center rounded-b-lg
      {$page.url.pathname === '/extrusion' ? 'bg-neutral-500' : 'bg-neutral-600'}       
      px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Extr
    </a>
  {/if}
  <div class="flex flex-grow items-end justify-end">
    <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
  </div>
  <button
    on:click|preventDefault="{() => (confirmKill = true)}"
    class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-red-500 disabled:opacity-50">
    Kill
  </button>
</div>
{#if confirmKill}
  <div class="absolute flex h-full w-full items-center justify-center bg-black bg-opacity-50">
    <div class="flex flex-col items-center justify-center gap-4 rounded-lg border-neutral-600 bg-neutral-700 bg-opacity-50 p-4 drop-shadow-md backdrop-blur">
      <p class=" text-center text-neutral-100">Are you sure you want to emergency stop the printer?</p>
      <span class="flex w-1/2 gap-3">
        <button
          on:click|preventDefault="{() => commands.emergencyStop()}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-red-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-neutral-500 disabled:opacity-50">
          Kill
        </button>
        <button
          on:click|preventDefault="{() => (confirmKill = false)}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Abort
        </button>
      </span>
    </div>
  </div>
{/if}
