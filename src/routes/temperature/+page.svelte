<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, clockFormatter, clock, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  let printStatsState = moonraker.printStats.State;
  let heaterBedTargetTemperature = moonraker.heaterBed.Target;
  let nozzleTargetTemperature = moonraker.extruder.Target;
  let heaterBedCurrentTemperature = moonraker.heaterBed.Temperature;
  let nozzleCurrentTemperature = moonraker.extruder.Temperature;

  let stepsArr = [1, 5, 10, 20, 50, 100];
  let selectedStep = 3;

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
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

  async function changeNozzleTemperature(relativeSteps: number) {
    let temp = getAbsoluteNozzleTemperature(relativeSteps);

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
  <div class="flex w-full flex-row items-center justify-end gap-3 p-1">
    <p class="text-sm text-neutral-50">M117 Status Message</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-evenly">
      <div class="flex w-36 flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
        <table class="self-stretch text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="px-2 text-end">Nozzle</td>

            <td class=" text-start">{$nozzleCurrentTemperature.toFixed(1)} °C</td>
          </tr>
          <tr>
            <td class="px-2 text-end">Target</td>

            <td class=" text-start">{$nozzleTargetTemperature.toFixed(1)} °C</td>
          </tr>
        </table>

        <button
          on:click="{() => changeNozzleTemperature(stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Up
        </button>

        <button
          on:click|preventDefault="{() => changeNozzleTemperature(-stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Down
        </button>
      </div>
      <div class="flex w-36 flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
        <table class="self-stretch text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="px-2 text-end">Bed</td>
            <td class="text-start">{$heaterBedCurrentTemperature.toFixed(1)} °C</td>
          </tr>
          <tr>
            <td class="px-2 text-end">Target</td>
            <td class="text-start">{$heaterBedTargetTemperature.toFixed(1)} °C</td>
          </tr>
        </table>

        <button
          on:click|preventDefault="{() => changeBedTemperature(stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Up
        </button>

        <button
          on:click|preventDefault="{() => changeBedTemperature(-stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Down
        </button>
      </div>
    </div>
    <span class="flex flex-col">
      <span class="flex flex-grow flex-col justify-start gap-2">
        <button
          class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
          on:click|preventDefault="{() => {
            disableNozzleTemperature();
            disableBedTemperature();
          }}">
          Off
        </button>
      </span>
      <span class="flex flex-grow flex-col justify-end gap-2">
        <button
          disabled="{true}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Preset
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
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      href="/printstate">
      State
    </a>
    {#if $printStatsState !== 'printing'}
      <a
        href="/move"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Move
      </a>
    {/if}
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Temp
    </button>
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
      on:click|preventDefault="{emergencyStop}">
      Kill
    </button>
  </div>
</div>
