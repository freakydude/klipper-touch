<script lang="ts">
  import { commands, moonraker, values } from '$lib/base.svelte';
  import BottomNavigation from '$lib/BottomNavigation.svelte';
  import StatusLine from '$lib/StatusLine.svelte';

  let printStatsState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;
  let gcodeMoveHomingOrigin = moonraker.gcodeMove.HomeOrigin;

  let stepsArr = [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5];
  let selectedStep = 3;

  let valuesStepsBaby = values.stepsBaby;

  $: {
    let stepIdx = stepsArr.indexOf($valuesStepsBaby);
    if (stepIdx != -1) {
      selectedStep = stepIdx;
    } else {
      console.warn("Global stepsBaby can't be pre-selected");
    }
  }

  let isHomedXY = false;
  let isHomedZ = false;

  $: {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  }
</script>

<div class="page-dark flex-col items-stretch justify-between">
  <StatusLine />
  <div class="flex h-full flex-row">
    <div class="flex w-5/6 items-center justify-around">
      <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-700 p-1">
        <table class="text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="px-2 text-end">Z</td>
            <td class=" text-start">{$motionReportLivePosition[2].toFixed(3)} mm</td>
          </tr>
          <tr>
            <td class="px-2 text-end">Z Offset</td>
            <td class=" text-start">{$gcodeMoveHomingOrigin[2].toFixed(3)} mm</td>
          </tr>
        </table>
        <span class="flex items-center gap-x-6">
          <span class="flex flex-col gap-3">
            <button
              on:click="{() => commands.changeOffset(stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Up
            </button>

            <button
              on:click="{() => commands.changeOffset(-stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Down
            </button></span>

          <button
            on:click="{() => commands.resetOffset()}"
            class="flex h-10 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Reset
          </button>
        </span>
      </div>
    </div>
    <span class="flex w-1/6 flex-col">
      <span class="flex flex-grow flex-col justify-around gap-2">
        <button
          disabled="{$printStatsState === 'printing'}"
          class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
          on:click="{() => commands.homeZ(isHomedXY)}">
          HomeZ
        </button>
        <button
          disabled="{!isHomedZ || $printStatsState === 'printing'}"
          on:click="{() => commands.moveAbsolute(undefined, undefined, 0)}"
          class="flex h-10 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Z = 0
        </button>

        <button
          disabled="{$gcodeMoveHomingOrigin[2] == 0 || $printStatsState === 'printing'}"
          on:click="{() => commands.saveConfig()}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
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
            : 'bg-neutral-600'}"
          on:click="{() => {
            selectedStep = i;
            $valuesStepsBaby = number;
          }}">
          {number}
        </button>
      {/each}
    </div>
  </span>
  <BottomNavigation />
</div>
