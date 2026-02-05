<script lang="ts">
  import { run } from 'svelte/legacy';

  import { commands, moonraker, values } from '$lib/base.svelte';
  import BottomNavigation from '$lib/BottomNavigation.svelte';
  import StatusLine from '$lib/StatusLine.svelte';

  let printState = moonraker.printStats.State;
  let liveExtruderVelocity = moonraker.motionReport.LiveExtruderVelocity;
  let nozzleTarget = moonraker.extruder.Target;
  let nozzleTemp = moonraker.extruder.Temperature;
  let canExtrude = moonraker.extruder.CanExtrude;

  let stepsArr = [1, 2, 5, 10, 20, 50, 100];
  let selectedStep = $state(3);
  let speedArr = [1, 2, 3, 5, 7, 10];
  let selectedSpeed = $state(2);

  let valuesStepsExtrusion = values.stepsExtrusion;

  run(() => {
    let stepIdx = stepsArr.indexOf($valuesStepsExtrusion);
    if (stepIdx != -1) {
      selectedStep = stepIdx;
    } else {
      console.warn("Global stepsExtrusion can't be pre-selected");
    }
  });

  let valuesStepsExtrusionSpeed = values.stepsExtrusionSpeed;

  run(() => {
    let stepIdx = stepsArr.indexOf($valuesStepsExtrusionSpeed);
    if (stepIdx != -1) {
      selectedSpeed = stepIdx;
    } else {
      console.warn("Global stepsExtrusionSpeed can't be pre-selected");
    }
  });
</script>

<div class="flex flex-grow flex-col items-stretch justify-between gap-1 overflow-hidden bg-neutral-800">
  <StatusLine />
  <div class="flex h-full flex-row">
    <div class="flex w-5/6 items-center justify-around gap-1">
      <div class="flex flex-col items-center gap-1 rounded-lg bg-neutral-700 p-1">
        <table class="w-40 self-stretch text-sm text-neutral-50">
          <tbody>
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="text-start">{$nozzleTemp.toFixed(1)} / {$nozzleTarget} °C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Flow</td>
              <td class="text-start">{(Math.pow(1.75 / 2, 2) * Math.PI * $liveExtruderVelocity).toFixed(1)} mm³/s</td>
            </tr>
          </tbody>
        </table>

        <button
          disabled={$printState === 'printing' || $canExtrude === false}
          onclick={() => commands.extrude(-stepsArr[selectedStep], speedArr[selectedSpeed])}
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
        >
          Retract
        </button>

        <button
          disabled={$printState === 'printing' || $canExtrude === false}
          onclick={() => commands.extrude(stepsArr[selectedStep], speedArr[selectedSpeed])}
          class="flex h-14 w-full items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
        >
          Extrude
        </button>
      </div>

      <span class="flex flex-col items-center justify-center">
        <div class="flex w-40 flex-col items-center gap-1 rounded-lg bg-neutral-700 p-1">
          <p class="flex text-neutral-50">Speed</p>
          <span class="flex flex-wrap justify-center gap-1">
            {#each speedArr as number, i}
              <button
                class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
                selectedSpeed
                  ? 'bg-neutral-500'
                  : 'bg-neutral-600'} "
                onclick={() => {
                  selectedSpeed = i;
                  $valuesStepsExtrusionSpeed = number;
                }}
              >
                {number}
              </button>
            {/each}
          </span>
        </div>
      </span>
    </div>

    <span class="flex w-1/6 flex-col justify-around gap-2">
      <!-- Right side buttons here -->

      <button
        disabled={true}
        class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      >
        Load
      </button>
      <button
        disabled={true}
        class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      >
        Unload
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
            $valuesStepsExtrusion = number;
          }}
        >
          {number}
        </button>
      {/each}
    </div>
  </span>
  <BottomNavigation />
</div>
