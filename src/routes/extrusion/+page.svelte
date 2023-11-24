<script lang="ts">
  import { commands, moonraker, values } from '$lib/base.svelte';

  let printState = moonraker.printStats.State;
  let liveExtruderVelocity = moonraker.motionReport.LiveExtruderVelocity;
  let nozzleTarget = moonraker.extruder.Target;
  let nozzleTemp = moonraker.extruder.Temperature;
  let canExtrude = moonraker.extruder.CanExtrude;
  let displayStatusMessage = moonraker.displayStatus.Message;

  let clockFormatter = values.clockFormatter;
  let clock = values.clock;

  let stepsArr = [1, 2, 5, 10, 20, 50, 100];
  let selectedStep = 3;
  let speedArr = [1, 2, 3, 5, 7, 10];
  let selectedSpeed = 2;
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex h-6 w-full flex-row items-center justify-center gap-1">
    <p class="text-sm text-neutral-50">{$displayStatusMessage}</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-evenly">
      <div class="flex flex-row items-center gap-3">
        <div class="flex w-44 flex-col items-center gap-2 rounded-lg bg-neutral-700 px-2 py-2">
          <table class="self-stretch text-sm text-neutral-50">
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="text-start">{$nozzleTemp.toFixed(1)} / {$nozzleTarget} °C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Flow</td>
              <td class="text-start">{(Math.pow(1.75 / 2, 2) * Math.PI * $liveExtruderVelocity).toFixed(1)} mm³/s</td>
            </tr>
          </table>

          <button
            disabled="{$printState === 'printing' || $canExtrude === false}"
            on:click|preventDefault="{() => commands.extrude(-stepsArr[selectedStep], speedArr[selectedSpeed])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Retract
          </button>

          <button
            disabled="{$printState === 'printing' || $canExtrude === false}"
            on:click|preventDefault="{() => commands.extrude(stepsArr[selectedStep], speedArr[selectedSpeed])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Extrude
          </button>
        </div>

        <span class="flex flex-col items-center justify-center self-end">
          <div class="flex w-40 flex-col items-center gap-1 rounded-lg bg-neutral-700 p-1">
            <p class="flex text-neutral-50">Speed</p>
            <span class="flex flex-wrap justify-center gap-1">
              {#each speedArr as number, i}
                <button
                  class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
                  selectedSpeed
                    ? 'bg-neutral-500'
                    : 'bg-neutral-600'} "
                  on:click|preventDefault="{() => {
                    selectedSpeed = i;
                  }}">
                  {number}
                </button>
              {/each}
            </span>
          </div>
        </span>
      </div>
    </div>
    <span class="flex flex-col">
      <span class="flex flex-grow flex-col justify-start gap-2">
        <!-- Right side buttons here -->
        <span class="flex flex-grow flex-col justify-end gap-2">
          <button
            disabled="{true}"
            class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Preset
          </button>
        </span>
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
    {#if $printState !== 'printing'}
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
    <a
      href="/babysteps"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Baby
    </a>
    {#if $printState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Extr
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      on:click|preventDefault="{()=>commands.emergencyStop()}">
      Kill
    </button>
  </div>
</div>
