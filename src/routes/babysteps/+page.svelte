<script lang="ts">
  import { commands, moonraker, values } from '$lib/base.svelte';

  let printStatsState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;
  let gcodeMoveHomingOrigin = moonraker.gcodeMove.HomeOrigin;
  let displayStatusMessage = moonraker.displayStatus.Message;

  let clockFormatter = values.clockFormatter;
  let clock = values.clock;

  let stepsArr = [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5];
  let selectedStep = 3;

  let isHomedXY = false;
  let isHomedZ = false;

  $: {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex w-full flex-row items-center justify-center gap-3 p-1">
    <p class="text-sm text-neutral-50">{$displayStatusMessage}</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-evenly">
      <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
        <table class="self-stretch text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="px-2 text-end">Z</td>
            <td class=" text-start">{$motionReportLivePosition[2].toFixed(3)} mm</td>
          </tr>
          <tr>
            <td class="px-2 text-end">Z Offset</td>
            <td class=" text-start">{$gcodeMoveHomingOrigin[2].toFixed(3)} mm</td>
          </tr>
        </table>
        <span class="flex items-center gap-2">
          <span class="flex flex-col gap-3">
            <button
              on:click|preventDefault="{() => commands.changeOffset(stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Up
            </button>

            <button
              on:click|preventDefault="{() => commands.changeOffset(-stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Down
            </button></span>

          <button
            on:click|preventDefault="{commands.resetOffset}"
            class="flex h-10 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Reset
          </button>
        </span>
      </div>
    </div>
    <span class="flex flex-col">
      <span class="flex flex-grow flex-col justify-start gap-2">
        <button
          disabled="{$printStatsState === 'printing'}"
          class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
          on:click|preventDefault="{() => commands.homeZ(isHomedXY)}">
          HomeZ
        </button>
        <button
          disabled="{!isHomedZ || $printStatsState === 'printing'}"
          on:click|preventDefault="{()=>commands.moveAbsolute(undefined,undefined,0)}"
          class="flex h-10 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Z = 0
        </button>
      </span>
      <span class="flex flex-grow flex-col justify-end gap-2">
        <button
          disabled="{$gcodeMoveHomingOrigin[2] == 0 || $printStatsState === 'printing'}"
          on:click|preventDefault="{commands.saveConfig}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Save
        </button>
      </span>
    </span>
  </div>
  <span class="flex flex-grow flex-row items-center justify-center py-2">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
          selectedStep
            ? 'bg-neutral-500'
            : 'bg-neutral-600'} "
          on:click|preventDefault="{() => {
            selectedStep = i;
          }}">
          {number}
        </button>
      {/each}
    </div>
  </span>
  <!-- Nav -->
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <a
      href="/printstate"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      State
    </a>
    {#if $printStatsState !== 'printing'}
      <a
        href="/move"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Move
      </a>
    {/if}
    <a
      href="/temperature"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Temp
    </a>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Baby
    </button>
    {#if $printStatsState !== 'printing'}
      <a
        href="/extrusion"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Extr
      </a>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      on:click|preventDefault="{commands.emergencyStop}">
      Kill
    </button>
  </div>
</div>
