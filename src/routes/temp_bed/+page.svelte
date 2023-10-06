<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { faArrowDown, faArrowUp, faBarsProgress, faGear, faList, faMinus, faPause, faPlus, faSkull, faStop } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let stepsArrIdx = 4;
  let stepsArr = [1, 2, 5, 10, 20, 50, 100];
  let presetArr = [0, 50, 60, 70, 90, 110];

  let nozzleTemp = moonraker.extruder.Temperature;
  let nozzleTempTarget = moonraker.extruder.Target;

  let bedTemp = moonraker.heaterBed.Temperature;
  let bedTempTarget = moonraker.heaterBed.Target;

  let distance = stepsArr[stepsArrIdx];

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }

  function increaseDistance() {
    stepsArrIdx = Math.min(stepsArrIdx + 1, stepsArr.length - 1);
    distance = stepsArr[stepsArrIdx];
  }

  function decreaseDistance() {
    stepsArrIdx = Math.max(stepsArrIdx - 1, 0);
    distance = stepsArr[stepsArrIdx];
  }

  async function changeNozzleTemperature(tempSteps: number) {
    let temperature = Math.max(0, $nozzleTempTarget + tempSteps);

    let nozzleTempRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M104 S' + temperature
      }
    });
    await client.sendRequest(nozzleTempRequest);
  }

  async function changeBedTemperature(tempSteps: number) {
    let temperature = Math.max(0, $bedTempTarget + tempSteps);

    let bedTempRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M140 S' + temperature
      }
    });
    await client.sendRequest(bedTempRequest);
  }
</script>

<div class="flex w-full flex-col flex-wrap justify-between gap-2 border-t-2 border-blue-400 bg-neutral-800 p-1">
  <div class="flex flex-col flex-wrap items-start justify-start gap-1">
    <p class="flex border-neutral-600 text-sm text-white">Preset in °C</p>
    <div class="flex flex-row flex-wrap gap-1">
      {#each presetArr as number, i}
        <button
          class="flex w-11 rounded border-l-4 border-neutral-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          on:click={() => setIdxDistance(i)}
        >
          <!-- <div class="self-center px-1"><Fa icon={faArrowLeft} /></div> -->
          <div class="flex-grow self-center text-end">{number}</div>
        </button>
      {/each}
    </div>
  </div>
  <div class="flex flex-col flex-wrap items-start justify-start">
    <div class="flex flex-col flex-wrap">
      <div class="flex justify-end border-neutral-600 text-sm text-white">
        <p class="px-1 text-blue-400">Current</p>
        <p class="flex-grow pl-2 pr-1 text-end">{$bedTemp.toFixed(0)} °C</p>
      </div>
      <div class="flex justify-end border-neutral-600 text-sm text-white">
        <p class="px-1 text-blue-400">Target</p>
        <p class="flex-grow pl-2 pr-1 text-end">{$bedTempTarget.toFixed(0)} °C</p>
      </div>
    </div>
    <div class="flex flex-col flex-wrap items-stretch justify-start gap-1 pt-1">
      <button
        class="flex rounded border-l-4 border-blue-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
        on:click={() => changeBedTemperature(distance)}
      >
        <div class="self-center px-1"><Fa icon={faArrowUp} /></div>
        <div class="flex-grow pl-2 pr-1 text-end">Increase</div>
      </button>
      <button
        class="flex rounded border-l-4 border-blue-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
        on:click={() => changeBedTemperature(-distance)}
      >
        <div class="self-center px-1"><Fa icon={faArrowDown} /></div>
        <div class="flex-grow pl-2 pr-1 text-end">Decrease</div>
      </button>
    </div>
  </div>
  <div class="flex flex-col flex-wrap items-start justify-start gap-1">
    <p class="flex border-neutral-600 text-sm text-white">Steps in °C</p>
    <div class="flex flex-row flex-wrap gap-1">
      {#each stepsArr as number, i}
        <button
          class="flex w-11 rounded border-l-4 border-neutral-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          on:click={() => setIdxDistance(i)}
        >
          <!-- <div class="self-center px-1"><Fa icon={faArrowLeft} /></div> -->
          <div class="flex-grow self-center text-end">{number}</div>
        </button>
      {/each}
    </div>
  </div>

  <div class="flex flex-grow flex-col flex-wrap justify-center gap-1">
    <button
      class=" flex rounded border-l-4 border-green-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
      on:click={() => goto('/')}
    >
      <div class="self-center px-2"><Fa icon={faBarsProgress} /></div>
      <div class="flex-grow self-center pl-1 pr-2 text-end">Status</div>
    </button>
  </div>
</div>
