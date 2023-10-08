<script lang="ts">
  import { Toolhead } from './../../lib/moonraker/modules/Toolhead.ts';
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { faArrowDown, faArrowUp, faArrowsUpDownLeftRight, faBarsProgress, faGear, faList, faMinus, faPlus, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let zOffset = moonraker.gcodeMove.HomeOrigin;
  let stepsArrIdx = 4;
  let stepsArr = [0.001, 0.002, 0.005, 0.01, 0.02, 0.05, 0.1];

  let distance = stepsArr[stepsArrIdx];

  async function changeZOffset(offsetSteps: number) {
    let zAdjustRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z_ADJUST=' + offsetSteps + ' MOVE=1'
      }
    });
    await client.sendRequest(zAdjustRequest);
  }

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

  async function saveOffset() {
    // TODO
  }

  async function clearOffset() {
    let zAdjustRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z=0' + ' MOVE=1'
      }
    });
    await client.sendRequest(zAdjustRequest);
  }
</script>

<div class="flex flex-grow flex-row border-t border-pink-400 bg-neutral-800">
  <div class="flex w-3/4 flex-col items-center justify-around gap-1 p-1">
    <span class="flex">
      <div class="flex flex-col items-stretch gap-1 p-1">
        <div class="flex text-sm text-white">
          <p class="px-1 text-pink-400">Z</p>
          <p class="flex-grow pl-2 pr-1 text-end">{$zOffset.toFixed(3)} mm</p>
        </div>
        <button class="flex rounded border-l-4 border-pink-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500">
          <div class="self-center px-1"><Fa icon={faArrowUp} /></div>
          <div class="flex-grow pl-2 pr-1 text-end">Up</div></button
        >
        <button class="flex rounded border-l-4 border-pink-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500">
          <div class="self-center px-1"><Fa icon={faArrowDown} /></div>
          <div class="flex-grow pl-2 pr-1 text-end">Down</div></button
        >
      </div>
    </span>
    <span class="flex">
      <div class="flex flex-col gap-1 p-1">
        <p class="flex border-neutral-600 text-sm text-white">Step in mm</p>
        <div class="flex flex-row flex-wrap gap-1">
          {#each stepsArr as number, i}
            <button
              class="flex w-1/5 rounded border-l-4 border-neutral-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
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
        class="flex rounded border-l-4 border-purple-400 bg-neutral-700 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        on:click={() => goto('/toolhead')}
      >
        <div class="self-center px-2"><Fa icon={faArrowsUpDownLeftRight} /></div>
        <div class="flex-grow self-center pl-1 pr-2 text-end">Toolhead</div>
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
