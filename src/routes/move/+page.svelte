<script lang="ts">
  import { commands, moonraker, values } from '$lib/base.svelte';

  let printStatsState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadAxisMaximum = moonraker.toolhead.AxisMaximum;
  let toolheadAxisMinimum = moonraker.toolhead.AxisMinimum;
  let toolheadPosition = moonraker.toolhead.Position;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;
  let displayStatusMessage = moonraker.displayStatus.Message;

  let clockFormatter = values.clockFormatter;
  let clock = values.clock;

  let stepsArr = [0.1, 1, 2, 5, 10, 20, 50, 100];
  let selectedStep = 6;

  let isHomedXY = false;
  let isHomedZ = false;

  $: {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  }

  function getAbsolutePosition(axis: number, relativeSteps: number): number {
    let position = $toolheadPosition[axis] + relativeSteps;
    position = Math.max(position, $toolheadAxisMinimum[axis]);
    position = Math.min(position, $toolheadAxisMaximum[axis]);

    return position;
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex w-full flex-row items-center justify-center gap-3 p-1">
    <p class="text-sm text-neutral-50">{$displayStatusMessage}</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-center gap-2">
      <div class="flex flex-col items-stretch justify-center gap-2 rounded-lg bg-neutral-600 p-1">
        <table class="self-center text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-4 text-end">Current</td>
            <td class="pr-3 text-start">X {$motionReportLivePosition[0].toFixed(2)}</td>
            <td class="pr-1 text-start">Y {$motionReportLivePosition[1].toFixed(2)}</td>
          </tr>
          <tr>
            <td class="pr-4 text-end">Target</td>
            <td class="pr-3 text-start">X {$toolheadPosition[0].toFixed(1)}</td>
            <td class="pr-1 text-start">Y {$toolheadPosition[1].toFixed(1)}</td>
          </tr>
        </table>

        <div class="flex items-center gap-2">
          <button
            disabled="{!isHomedXY}"
            on:click|preventDefault="{() => commands.moveAbsolute(getAbsolutePosition(0, -stepsArr[selectedStep]), undefined, undefined)}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Left
          </button>

          <span class="flex flex-col gap-3">
            <button
              disabled="{!isHomedXY}"
              on:click|preventDefault="{() => commands.moveAbsolute(undefined, getAbsolutePosition(1, +stepsArr[selectedStep]), undefined)}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Back
            </button>
            <button
              disabled="{!isHomedXY}"
              on:click|preventDefault="{() => commands.moveAbsolute(undefined, getAbsolutePosition(1, -stepsArr[selectedStep]), undefined)}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Front
            </button>
          </span>

          <button
            disabled="{!isHomedXY}"
            on:click|preventDefault="{() => commands.moveAbsolute(getAbsolutePosition(0, +stepsArr[selectedStep]), undefined, undefined)}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Right
          </button>
        </div>
      </div>

      <div class="flex flex-col items-stretch justify-center gap-2 rounded-lg bg-neutral-600 p-1">
        <table class="self-center text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-1 text-start">Z {$motionReportLivePosition[2].toFixed(2)}</td>
          </tr>
          <tr>
            <td class="pr-1 text-start">Z {$toolheadPosition[2].toFixed(1)}</td>
          </tr>
        </table>

        <span class="flex flex-col items-center justify-center gap-3">
          <button
            disabled="{!isHomedZ}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
            on:click|preventDefault="{() => commands.moveAbsolute(undefined, undefined, getAbsolutePosition(2, -stepsArr[selectedStep]))}">
            Down
          </button>

          <button
            disabled="{!isHomedZ}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
            on:click|preventDefault="{() => commands.moveAbsolute(undefined, undefined, getAbsolutePosition(2, +stepsArr[selectedStep]))}">
            Up
          </button>
        </span>
      </div>
    </div>
    <span class="flex flex-col">
      <button
        on:click|preventDefault="{commands.disableSteppers}"
        class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Off
      </button>
      <span class="flex flex-grow flex-col justify-end gap-3">
        <button
          on:click|preventDefault="{commands.homeXY}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg {isHomedXY
            ? 'bg-neutral-500'
            : 'bg-neutral-700'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          HomeXY
        </button>
        <button
          on:click|preventDefault="{() => commands.homeZ(isHomedXY)}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg {isHomedZ
            ? 'bg-neutral-500'
            : 'bg-neutral-700'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          HomeZ
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
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <a
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      href="/printstate">
      State
    </a>
    {#if $printStatsState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Move
      </button>
    {/if}
    <a
      href="/temperature"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Temp
    </a>
    <a
      href="/babysteps"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Baby
    </a>
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
