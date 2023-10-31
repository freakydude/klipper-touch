<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  let printStatsState = moonraker.printStats.State;
  let heaterBedTargetTemperature = moonraker.heaterBed.Target;
  let nozzleTargetTemperature = moonraker.extruder.Target;
  let heaterBedCurrentTemperature = moonraker.heaterBed.Temperature;
  let nozzleCurrentTemperature = moonraker.extruder.Temperature;

  let localTime: string = calcClock();

  let stepsArr = [1, 5, 10, 20, 50, 100];
  let selectedStep = 3;

  function calcClock(): string {
    return new Date(Date.now()).toLocaleTimeString('de', { timeStyle: 'short' });
  }

  setInterval(() => {
    localTime = calcClock();
  }, 1000);

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }

  function getAbsoluteNozzleTemperature(relativeSteps: number): number {
    let position = $nozzleTargetTemperature + relativeSteps;
    position = Math.max(position, 0);
    position = Math.min(position, 500);

    return position;
  }

  function getAbsoluteBedTemperature(relativeSteps: number): number {
    let position = $heaterBedTargetTemperature + relativeSteps;
    position = Math.max(position, 0);
    position = Math.min(position, 150);

    return position;
  }

  async function changeNozzleTemperature(relativeSteps: number) {
    let temp = getAbsoluteNozzleTemperature(relativeSteps);
    console.log(temp);
    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M104 S' + temp
      }
    });
    await client.sendRequest(request);
  }

  async function disableNozzleTemperature() {
    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M104 S0'
      }
    });
    await client.sendRequest(request);
  }

  async function changeBedTemperature(relativeSteps: number) {
    let temp = getAbsoluteBedTemperature(relativeSteps);

    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M140 S' + temp
      }
    });
    await client.sendRequest(request);
  }

  async function disableBedTemperature() {
    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M140 S0'
      }
    });
    await client.sendRequest(request);
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex flex-row justify-center">
    <p class=" text-sm text-neutral-50">Temp Screen Status</p>
  </div>
  <div class="flex flex-grow flex-row">
    <div class="flex flex-grow flex-col">
      <div class="flex flex-grow flex-col items-center justify-center">
        <div class="flex flex-col items-center gap-1">
          <p class="flex flex-row text-center text-sm text-neutral-50">
            Nozzle {$nozzleCurrentTemperature.toFixed(1)} / {$nozzleTargetTemperature.toFixed(0)} °C
          </p>
          <div class="flex items-center gap-3">
            <button
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
              on:click="{() => disableNozzleTemperature()}">
              Off
            </button>
            <button
              on:click="{() => changeNozzleTemperature(-stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
              Less
            </button>
            <button
              on:click="{() => changeNozzleTemperature(stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
              More
            </button>
          </div>
          <p class="flex flex-row text-center text-sm text-neutral-50">
            Bed {$heaterBedCurrentTemperature.toFixed(1)} / {$heaterBedTargetTemperature.toFixed(0)} °C
          </p>
          <div class="flex items-center gap-3">
            <button
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
              on:click="{() => disableBedTemperature()}">
              Off
            </button>
            <button
              on:click="{() => changeBedTemperature(-stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
              Less
            </button>
            <button
              on:click="{() => changeBedTemperature(stepsArr[selectedStep])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
              More
            </button>
          </div>
        </div>
      </div>
    </div>
    <div class="flex flex-col justify-center">
      <button
        disabled="{true}"
        class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Preset
      </button>
    </div>
  </div>

  <!-- Statusbar -->
  <!-- <div class="flex w-full flex-row items-center justify-center gap-3">
    <p class="text-center text-sm text-neutral-50">
      Nozzle {$nozzleCurrentTemperature.toFixed(1)} °C
    </p>
    <p class="text-center text-sm text-neutral-50">
      Bed {$heaterBedCurrentTemperature.toFixed(1)} °C
    </p>
  </div> -->
  <!-- Temperature -->
  <!-- <div class="flex flex-grow flex-row justify-between">
    <div class="flex flex-grow flex-row justify-center gap-3">
      <span class="flex flex-col items-center justify-center gap-3">
        <p class="flex text-sm text-neutral-50">Off</p>
        <button
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
          on:click="{() => disableNozzleTemperature()}">
          NOff
        </button>
        <button
          on:click="{() => disableBedTemperature()}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          BOff
        </button>
      </span>

      <span class="flex flex-col items-center justify-center gap-3">
        <p class="flex text-sm text-neutral-50">N {$nozzleTargetTemperature.toFixed(0)} °C</p>
        <button
          on:click="{() => changeNozzleTemperature(-stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          NLess
        </button>
        <button
          on:click="{() => changeBedTemperature(-stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          BLess
        </button>
      </span>

      <span class="flex flex-col items-center justify-center gap-3">
        <p class="flex text-sm text-neutral-50">B {$heaterBedTargetTemperature.toFixed(0)} °C</p>
        <button
          on:click="{() => changeNozzleTemperature(stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          NMore
        </button>

        <button
          on:click="{() => changeBedTemperature(stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          BMore
        </button>
      </span>
    </div>
    <span class="flex flex-col gap-3">
      <span class="flex flex-grow flex-col justify-center gap-3">
        <button
          disabled="{true}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          Preset
        </button>
      </span>
    </span>
  </div> -->
  <!-- Steps -->
  <span class="flex flex-row items-center justify-center py-2">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50 {i ===
          selectedStep
            ? 'bg-neutral-500'
            : 'bg-neutral-600'} "
          on:click="{() => {
            selectedStep = i;
          }}">
          {number}
        </button>
      {/each}
    </div>
  </span>
  <!-- Nav -->
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
      on:click="{async () => goto('/printstate')}">
      State
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        on:click="{async () => goto('/move')}"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Move
      </button>
    {/if}
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
      Temp
    </button>
    <button
      disabled="{true}"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
      Baby
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        disabled="{true}"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Prep
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{localTime}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
      on:click="{emergencyStop}">
      Kill
    </button>
  </div>
</div>
