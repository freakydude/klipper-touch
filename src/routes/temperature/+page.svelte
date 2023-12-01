<script lang="ts">
  import { commands, moonraker, values } from '$lib/base.svelte';
  import BottomNavigation from '$lib/BottomNavigation.svelte';
  import StatusLine from '$lib/StatusLine.svelte';

  let heaterBedTargetTemperature = moonraker.heaterBed.Target;
  let nozzleTargetTemperature = moonraker.extruder.Target;
  let heaterBedCurrentTemperature = moonraker.heaterBed.Temperature;
  let nozzleCurrentTemperature = moonraker.extruder.Temperature;
  let stepsArr = [1, 5, 10, 20, 50, 100];
  let selectedStep = 3;
  let preselectDialog = false;

  let valuesStepsTemp = values.stepsTemp;

  $: {
    let stepIdx = stepsArr.indexOf($valuesStepsTemp);
    if (stepIdx != -1) {
      selectedStep = stepIdx;
    } else {
      console.warn("Global stepsTemp can't be pre-selected");
    }
  }

  function getAbsoluteNozzleTemperature(relativeSteps: number): number {
    let target = $nozzleTargetTemperature + relativeSteps;
    target = Math.max(target, 0);
    target = Math.min(target, 500);

    return target;
  }

  function getAbsoluteBedTemperature(relativeSteps: number): number {
    let target = $heaterBedTargetTemperature + relativeSteps;
    target = Math.max(target, 0);
    target = Math.min(target, 150);

    return target;
  }
</script>

<div class="page-dark flex-col items-stretch justify-between gap-1">
  <StatusLine />
  <div class="flex h-full flex-row">
    <div class="flex w-5/6 items-center justify-around gap-1">
      <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-700 p-1">
        <table class="text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-1 text-end">Nozzle</td>
            <td class="w-16 text-start">{$nozzleCurrentTemperature.toFixed(1)} °C</td>
          </tr>
          <tr>
            <td class="pr-1 text-end">Target</td>
            <td class="w-16 text-start">{$nozzleTargetTemperature.toFixed(1)} °C</td>
          </tr>
        </table>

        <button
          on:click="{() => commands.setNozzleTemperature(getAbsoluteNozzleTemperature(stepsArr[selectedStep]))}"
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Up
        </button>

        <button
          on:click="{() => commands.setNozzleTemperature(getAbsoluteNozzleTemperature(-stepsArr[selectedStep]))}"
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Down
        </button>
      </div>
      <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-700 p-1">
        <table class=" text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-1 text-end">Bed</td>
            <td class="w-16 text-start">{$heaterBedCurrentTemperature.toFixed(1)} °C</td>
          </tr>
          <tr>
            <td class="pr-1 text-end">Target</td>
            <td class="w-16 text-start">{$heaterBedTargetTemperature.toFixed(1)} °C</td>
          </tr>
        </table>

        <button
          on:click="{() => commands.setBedTemperature(getAbsoluteBedTemperature(stepsArr[selectedStep]))}"
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Up
        </button>

        <button
          on:click="{() => commands.setBedTemperature(getAbsoluteBedTemperature(-stepsArr[selectedStep]))}"
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Down
        </button>
      </div>
    </div>
    <span class="flex w-1/6 flex-col justify-around gap-3">
      <button
        class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
        on:click|preventDefault="{() => {
          commands.setNozzleTemperature(0);
          commands.setBedTemperature(0);
        }}">
        Off
      </button>
      <button
        on:click|preventDefault="{() => (preselectDialog = true)}"
        class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Preset
      </button>
    </span>
  </div>
  <span class="flex flex-grow flex-row items-center justify-center py-1">
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
  <BottomNavigation />
</div>
{#if preselectDialog}
  <div class="absolute flex h-full w-full items-center justify-center bg-black bg-opacity-50 p-4">
    <div class="flex flex-col items-center justify-center gap-3 rounded-lg border-neutral-600 bg-neutral-700 bg-opacity-50 p-2 drop-shadow-md backdrop-blur">
      <p class="text-center text-neutral-100">Choose a Preset</p>
      <span class="flex flex-wrap justify-center gap-3">
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(200);
            commands.setBedTemperature(60);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>PLA</p>
            <p class="text-sm font-normal">200 / 60 °C</p>
          </span>
        </button>
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(235);
            commands.setBedTemperature(70);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>PETG</p>
            <p class="text-sm font-normal">235 / 70 °C</p>
          </span>
        </button>
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(250);
            commands.setBedTemperature(110);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>ABS</p>
            <p class="text-sm font-normal">250 / 110 °C</p>
          </span>
        </button>
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(220);
            commands.setBedTemperature(0);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>TPU</p>
            <p class="text-sm font-normal">220 / 0 °C</p>
          </span>
        </button>
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(170);
            commands.setBedTemperature(60);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>Heatup</p>
            <p class="text-sm font-normal">170 / 60 °C</p>
          </span>
        </button>
        <button
          on:click|preventDefault="{() => {
            commands.setNozzleTemperature(110);
            commands.setBedTemperature(0);
            preselectDialog = false;
          }}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          <span class="flex flex-col">
            <p>Coldpull</p>
            <p class="text-sm font-normal">110 / 0 °C</p>
          </span>
        </button>
      </span>
      <span class="flex flex-wrap justify-center gap-3">
        <button
          on:click|preventDefault="{() => (preselectDialog = false)}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-4 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Abort
        </button>
      </span>
    </div>
  </div>
{/if}
