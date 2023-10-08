<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import {
    faArrowDown,
    faArrowUp,
    faArrowsUpDownLeftRight,
    faBarsProgress,
    faDownLong,
    faGear,
    faList,
    faMinus,
    faPause,
    faPlus,
    faSkull,
    faSortDown,
    faSortUp,
    faStop
  } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let stepsArrIdx = 4;
  let stepsArr = [1, 2, 5, 10, 20, 50, 100];
  let presetArr = [1, 2, 3, 5, 7, 10];

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

<div class="flex flex-grow flex-row border-t border-teal-400 bg-neutral-800">
  <div class=" flex w-3/4 flex-col items-center justify-around px-1">
    <span class="flex flex-col justify-center gap-1">
      <button class="flex rounded border-l-4 border-teal-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500">
        <div class="self-center px-1"><Fa icon={faSortUp} /></div>
        <div class="flex-grow self-center px-1 text-end">Retract</div>
      </button>
      <button class="flex rounded border-l-4 border-teal-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500">
        <div class="self-center px-1"><Fa icon={faSortDown} /></div>
        <div class="flex-grow self-center px-1 text-end">Extrude</div>
      </button>
    </span>
    <span class="flex w-full items-center justify-center">
      <div class="flex flex-col gap-1">
        <p class="flex border-neutral-600 text-sm text-white">Steps in mm</p>
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
    </span>
    <span class="flex w-full items-center justify-center">
      <div class="flex flex-col gap-1">
        <p class="flex border-neutral-600 text-sm text-white">Speed in mm/s²</p>
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
    </span>
  </div>
  <div class="flex w-1/4 flex-col flex-wrap justify-around gap-1 px-1">
    <span class="flex flex-col flex-wrap justify-center gap-1">
      <button class="flex rounded border-l-4 border-red-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => goto('/temp_nozzle')}>
        <div class="self-center px-1"><Fa icon={faDownLong} /></div>
        <div class="flex-grow self-center pl-2 pr-1 text-end">{$nozzleTempTarget.toFixed(0)} °C</div>
      </button>

      <button
        class="flex rounded border-l-4 border-purple-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        on:click={() => goto('/toolhead')}
      >
        <div class="self-center px-2"><Fa icon={faArrowsUpDownLeftRight} /></div>
        <div class="flex-grow self-center pl-1 pr-2 text-end">Toolhead</div>
      </button>
    </span>
    <button
      class="flex rounded border-l-4 border-green-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
      on:click={() => goto('/')}
    >
      <div class="self-center px-2"><Fa icon={faBarsProgress} /></div>
      <div class="flex-grow self-center pl-1 pr-2 text-end">Status</div>
    </button>
    <button
      class="flex rounded border-l-4 border-red-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500 disabled:text-neutral-500"
      disabled
      on:click={() => emergencyStop()}
    >
      <div class="self-center px-1"><Fa icon={faSkull} /></div>
      <div class="flex-grow self-center pl-2 pr-1 text-end">Kill</div>
    </button>
  </div>
</div>
