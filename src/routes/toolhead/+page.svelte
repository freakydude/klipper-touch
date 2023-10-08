<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import {
    faArrowDown,
    faArrowLeft,
    faArrowRight,
    faArrowUp,
    faArrowsToDot,
    faArrowsUpDown,
    faArrowsUpDownLeftRight,
    faBarsProgress,
    faHome,
    faList,
    faMinus,
    faPlus,
    faSkull,
    faSort,
    faSquareFull
  } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let extruderSpeed = 5;
  let moveSpeed = 50;

  let toolheadPosition = moonraker.toolhead.Position;
  let nozzleTemp = moonraker.extruder.Temperature;

  let stepsArrIdx = 6;
  let stepsArr = [0.1, 0.5, 1, 5, 10, 50, 100];
  let speedArr = [0.5, 1, 2, 5, 10, 20, 50];

  let distance = stepsArr[stepsArrIdx];
  let isHomedXY = false;
  let isHomedZ = false;

  moonraker.toolhead.HomedAxes.subscribe((value) => {
    if (value.includes('xy')) {
      isHomedXY = true;
    } else {
      isHomedXY = false;
    }
    if (value.includes('z')) {
      isHomedZ = true;
    } else {
      isHomedZ = false;
    }
  });

  async function homeXY() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 X Y' // Home XY axies
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function homeXYZ() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28' // Home XY axies
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function homeZ() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 Z' // Home Z axies
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function moveRelative(x: number = 0, y: number = 0, z: number = 0) {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G91\nG0 X' + x + ' Y' + y + ' Z' + z + ' F' + moveSpeed * 60
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }

  async function extrudeRelative(e: number = 0) {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G91\n G1 E' + e + ' F' + extruderSpeed * 60
      }
    });
    await client.sendRequest(homeRequest);
  }

  function increaseDistance() {
    stepsArrIdx = Math.min(stepsArrIdx + 1, stepsArr.length - 1);
    distance = stepsArr[stepsArrIdx];
  }

  function decreaseDistance() {
    stepsArrIdx = Math.max(stepsArrIdx - 1, 0);
    distance = stepsArr[stepsArrIdx];
  }

  function setIdxDistance(idx: number = 0) {
    distance = stepsArr[idx];
  }
</script>

<div class="flex flex-grow flex-row border-t border-purple-400 bg-neutral-800">
  <div class="flex w-3/4 flex-col items-center justify-around gap-1 p-1">
    <span class="flex w-full flex-row justify-center">
      <div class="grid grid-cols-3 grid-rows-3 items-end justify-stretch gap-1">
        <div class="col-start-1 row-start-1 flex flex-grow flex-col flex-wrap">
          <div class="flex border-b-2 border-neutral-600 text-sm text-white">
            <p class="px-1 text-yellow-400">X</p>
            <p class="flex-grow pl-2 pr-1 text-end">{$toolheadPosition[0].toFixed(2)} mm</p>
          </div>
          <div class="flex border-neutral-600 text-sm text-white">
            <p class="px-1 text-yellow-400">Y</p>
            <p class="flex-grow pl-2 pr-1 text-end">{$toolheadPosition[1].toFixed(2)} mm</p>
          </div>
        </div>
        <button
          class="col-start-1 row-start-2 flex rounded border-l-4 border-yellow-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(0, -distance, 0)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowLeft} /></div>
          <p class="flex-grow self-center pl-1 pr-2 text-end">Left</p>
        </button>
        <div class="col-start-3 row-start-1 flex flex-grow flex-col flex-wrap">
          <div class="flex justify-end border-neutral-600 text-sm text-white">
            <p class="px-1 text-orange-400">Z</p>
            <p class="flex-grow pl-2 pr-1 text-end">{$toolheadPosition[2].toFixed(2)} mm</p>
          </div>
        </div>

        <button
          class="col-start-3 row-start-2 flex rounded border-l-4 border-yellow-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(0, distance, 0)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowRight} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Right</div>
        </button>
        <button
          class="col-start-2 row-start-1 flex rounded border-l-4 border-yellow-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(distance, 0, 0)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowUp} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Back</div>
        </button>
        <button
          class="col-start-2 row-start-2 flex rounded border-l-4 border-neutral-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={true}
        >
          <div class="self-center pl-2"><Fa icon={faArrowsToDot} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Center</div>
        </button>
        <button
          class="col-start-2 row-start-3 flex rounded border-l-4 border-yellow-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(-distance, 0, 0)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowDown} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Front</div>
        </button>

        <button
          class="col-start-1 row-start-3 flex rounded border-l-4 border-orange-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(0, 0, distance)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowUp} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Up</div>
        </button>

        <button
          class="col-start-3 row-start-3 flex rounded border-l-4 border-orange-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
          disabled={!isHomedXY}
          on:click={() => moveRelative(0, 0, -distance)}
        >
          <div class="self-center pl-2"><Fa icon={faArrowDown} /></div>
          <div class="flex-grow self-center pl-1 pr-2 text-end">Down</div>
        </button>
      </div>
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
          {#each speedArr as number, i}
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
  <div class="flex w-1/4 flex-col flex-wrap justify-around gap-1 p-1">
    <span class="flex flex-col flex-wrap justify-center gap-1">
      <button
        class=" flex rounded border-l-4 border-teal-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        on:click={() => goto('/extrude')}
      >
        <div class="self-center px-2"><Fa icon={faSort} /></div>
        <div class="flex-grow self-center pl-1 pr-2 text-end">Extrude</div>
      </button>
      <button
        class="flex rounded border-l-4 border-pink-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        on:click={() => goto('/zoffset')}
      >
        <div class="self-center px-2"><Fa icon={faArrowsUpDown} /></div>
        <div class="flex-grow self-center pl-1 pr-2 text-end">Z-Offset</div>
      </button>
    </span>
    <span class="flex flex-col flex-wrap justify-center gap-1">
      <button
        class="flex rounded border-l-4 border-green-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        on:click={() => goto('/')}
      >
        <div class="self-center px-2"><Fa icon={faBarsProgress} /></div>
        <div class="flex-grow self-center pl-1 pr-2 text-end">Status</div>
      </button>
    </span>
    <button
      class="flex rounded border-l-4 border-red-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500 disabled:text-neutral-500"
      disabled
      on:click={() => emergencyStop()}
    >
      <div class="self-center px-1"><Fa icon={faSkull} /></div>
      <div class="flex-grow pl-2 pr-1 text-end">Kill</div>
    </button>
  </div>
</div>
