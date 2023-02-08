<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import Fa from 'svelte-fa/src/fa.svelte';
  import { faArrowDown, faArrowUp, faList, faMinus, faPlus, faSkull } from '@fortawesome/free-solid-svg-icons';

  let zOffset = moonraker.gcodeZOffset;
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

<div class="flex flex-row bg-neutral-700 p-2">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch  flex flex-col bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch  flex flex-col bg-red-600" on:click={() => goto('/parameter')}>PA</button>
    <button class="btn-touch  flex flex-col bg-red-600" on:click={() => goto('/toolhead')}>TH</button>
    <button class="btn-touch  flex flex-col bg-blue-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col">
    <div class="flex grow flex-wrap content-center items-center justify-around  rounded">
      <div class="flex flex-row flex-wrap items-center gap-4">
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600">
            <p class="label">Z-Offset {$zOffset.toFixed(3)} mm</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-2 gap-1 p-1">
            <button class="btn-touch  col-start-1 row-start-1 " on:click={async () => changeZOffset(-distance)}><Fa icon={faArrowDown} /></button>
            <button class="btn-touch  col-start-2 row-start-1 " on:click={async () => changeZOffset(distance)}><Fa icon={faArrowUp} /></button>
            <button class="btn-touch col-start-1 row-start-2 px-3 py-5" on:click={saveOffset}>Save</button>
            <button class="btn-touch col-start-2 row-start-2 px-3 py-5" on:click={clearOffset}>Clear</button>
          </div>
        </div>
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600">
            <p class="label">Step {distance.toFixed(3)} mm</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1 ">
            <button class="btn-touch  col-start-1 row-start-1" on:click={increaseDistance}><Fa icon={faPlus} /></button>
            <button class="btn-touch  col-start-2 row-start-1" on:click={decreaseDistance}><Fa icon={faMinus} /></button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- <script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let zOffset = moonraker.gcodeZOffset;
  let zOffsetTarget = 0.0;
  let offsetSteps = 0.02;

  async function changeZOffset(offsetSteps: number) {
    zOffsetTarget += offsetSteps;

    let zAdjustRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z=' + zOffsetTarget
      }
    });
    await client.sendRequest(zAdjustRequest);
  }
</script>

<div class="flex grow flex-wrap items-center justify-evenly gap-2 bg-neutral-800">
  <div class="flex flex-col flex-wrap gap-1">
    <button class="btn-touch bg-red-900" on:click={() => goto('/')}>Menu</button>
  </div>

  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => changeZOffset(offsetSteps)}>{(zOffsetTarget + offsetSteps).toFixed(3)} mm</button>
    <p class="label">Z-Offset</p>
    <p class="label">{$zOffset.toFixed(3)} mm</p>
    <button class="btn-touch" on:click={async () => changeZOffset(-offsetSteps)}>{(zOffsetTarget - offsetSteps).toFixed(3)} mm</button>
  </div>

  <div class="flex flex-row flex-wrap items-center gap-1 bg-red-900 ">
    <button
      class="btn-touch h-16 w-24 {offsetSteps === 0.005 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.005;
      }}>0.005</button
    >
    <button
      class="btn-touch h-16 w-20 {offsetSteps === 0.01 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.01;
      }}>0.01</button
    >
    <button
      class="btn-touch h-16 w-20 {offsetSteps === 0.02 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.02;
      }}>0.02</button
    >
    <p class="label w-32">Z-Steps</p>
    <button
      class="btn-touch h-16 w-20 {offsetSteps === 0.05 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.05;
      }}>0.05</button
    >
    <button
      class="btn-touch h-16 w-20 {offsetSteps === 0.1 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.1;
      }}>0.10</button
    >
    <button
      class="btn-touch h-16 w-20 {offsetSteps === 0.02 ? 'selected' : ''}"
      on:click={() => {
        offsetSteps = 0.02;
      }}>0.02</button
    >
  </div>
</div>
 -->
