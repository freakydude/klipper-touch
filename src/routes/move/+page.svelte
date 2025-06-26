<script lang="ts">
  import { run } from 'svelte/legacy';

  import { commands, moonraker, values } from '$lib/base.svelte';
  import BottomNavigation from '$lib/BottomNavigation.svelte';
  import StatusLine from '$lib/StatusLine.svelte';

  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadAxisMaximum = moonraker.toolhead.AxisMaximum;
  let toolheadAxisMinimum = moonraker.toolhead.AxisMinimum;
  let toolheadPosition = moonraker.toolhead.Position;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;

  let stepsArr = [0.1, 1, 2, 5, 10, 20, 50, 100];
  let selectedStep = $state(6);
  let valuesStepsMove = values.stepsMove;

  run(() => {
    let stepIdx = stepsArr.indexOf($valuesStepsMove);
    if (stepIdx != -1) {
      selectedStep = stepIdx;
    } else {
      console.warn("Global stepsMove can't be pre-selected");
    }
  });

  let isHomedXY = $state(false);
  let isHomedZ = $state(false);

  run(() => {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  });

  function getAbsolutePosition(axis: number, relativeSteps: number): number {
    let position = $toolheadPosition[axis] + relativeSteps;
    position = Math.max(position, $toolheadAxisMinimum[axis]);
    position = Math.min(position, $toolheadAxisMaximum[axis]);

    return position;
  }
</script>

<div class="flex flex-grow flex-col items-stretch justify-between gap-1 overflow-hidden bg-neutral-800">
  <StatusLine />
  <div class="flex h-full flex-row">
    <div class="flex w-5/6 items-center justify-around gap-1">
      <div class="flex flex-col items-center justify-center gap-1 rounded-lg bg-neutral-700 p-1">
        <table class="text-sm text-neutral-50">
          <tbody>
            <tr class="border-b border-neutral-800">
              <td class="pr-3 text-end">Current</td>
              <td class="w-16 pr-2 text-start">X {$motionReportLivePosition[0].toFixed(2)}</td>
              <td class="w-16 text-start">Y {$motionReportLivePosition[1].toFixed(2)}</td>
            </tr>
            <tr>
              <td class="pr-3 text-end">Target</td>
              <td class="w-16 pr-2 text-start">X {$toolheadPosition[0].toFixed(1)}</td>
              <td class="w-16 text-start">Y {$toolheadPosition[1].toFixed(1)}</td>
            </tr>
          </tbody>
        </table>

        <div class="flex items-center gap-2">
          <button
            disabled={!isHomedXY}
            onclick={() => commands.moveAbsolute(getAbsolutePosition(0, -stepsArr[selectedStep]), undefined, undefined)}
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Left
          </button>

          <span class="flex flex-col gap-3">
            <button
              disabled={!isHomedXY}
              onclick={() => commands.moveAbsolute(undefined, getAbsolutePosition(1, +stepsArr[selectedStep]), undefined)}
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Back
            </button>
            <button
              disabled={!isHomedXY}
              onclick={() => commands.moveAbsolute(undefined, getAbsolutePosition(1, -stepsArr[selectedStep]), undefined)}
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Front
            </button>
          </span>

          <button
            disabled={!isHomedXY}
            onclick={() => commands.moveAbsolute(getAbsolutePosition(0, +stepsArr[selectedStep]), undefined, undefined)}
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Right
          </button>
        </div>
      </div>

      <div class="flex flex-col items-stretch justify-center gap-2 rounded-lg bg-neutral-700 p-1">
        <table class="self-center text-sm text-neutral-50">
          <tbody>
            <tr class="border-b border-neutral-800">
              <td class="pr-1 text-end">Z</td>
              <td class="w-10 text-start">{$motionReportLivePosition[2].toFixed(2)}</td>
            </tr>
            <tr>
              <td class="pr-1 text-end">Z</td>
              <td class="w-10 text-start">{$toolheadPosition[2].toFixed(1)}</td>
            </tr>
          </tbody>
        </table>

        <span class="flex flex-col items-center justify-center gap-3">
          <button
            disabled={!isHomedZ}
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
            onclick={() => commands.moveAbsolute(undefined, undefined, getAbsolutePosition(2, +stepsArr[selectedStep]))}>
            Up
          </button>
          <button
            disabled={!isHomedZ}
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
            onclick={() => commands.moveAbsolute(undefined, undefined, getAbsolutePosition(2, -stepsArr[selectedStep]))}>
            Down
          </button>
        </span>
      </div>
    </div>
    <span class="flex h-full w-1/6 flex-col justify-around gap-3">
      <button
        onclick={() => commands.disableSteppers()}
        class="flex h-10 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Off
      </button>
      <button
        onclick={() => commands.homeXY()}
        class="flex h-14 items-center justify-center rounded-l-lg {isHomedXY
          ? 'bg-neutral-500'
          : 'bg-neutral-600'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        HomeXY
      </button>
      <button
        onclick={() => commands.homeZ(isHomedXY)}
        class="flex h-14 items-center justify-center rounded-l-lg {isHomedZ
          ? 'bg-neutral-500'
          : 'bg-neutral-600'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        HomeZ
      </button>
    </span>
  </div>
  <span class="flex flex-row items-center justify-center py-1">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
          selectedStep
            ? 'bg-neutral-500'
            : 'bg-neutral-600'} "
          onclick={() => {
            selectedStep = i;
            $valuesStepsMove = number;
          }}>
          {number}
        </button>
      {/each}
    </div>
  </span>
  <BottomNavigation />
</div>
