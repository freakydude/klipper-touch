<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faArrowDown, faArrowUp, faList, faMinus, faPlus, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let zOffset = moonraker.gcodeMoveHomeOrigin;
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

<div class="flex flex-grow flex-row gap-1 bg-neutral-800 p-1">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/parameter')}>PA</button>
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/toolhead')}>TH</button>
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/printerstatus')}>PS</button>
    <div class="grow" />
    <button class="btn-touch flex flex-col bg-yellow-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex flex-grow flex-row flex-wrap items-center justify-around gap-4">
    <div class="flex flex-col rounded bg-neutral-600">
      <div class="flex flex-col items-stretch">
        <p class="label-head">Z-Offset</p>
        <p class="label">Current: {$zOffset.toFixed(3)} mm</p>
      </div>
      <div class="grid grid-cols-2 grid-rows-2 gap-1 p-1">
        <button class="btn-touch col-start-1 row-start-1" on:click={async () => changeZOffset(distance)}><Fa icon={faArrowUp} /></button>
        <button class="btn-touch col-start-2 row-start-1" on:click={async () => changeZOffset(-distance)}><Fa icon={faArrowDown} /></button>
        <button class="btn-touch col-start-1 row-start-2 px-3 py-5" on:click={saveOffset}>Save</button>
        <button class="btn-touch col-start-2 row-start-2 px-3 py-5" on:click={clearOffset}>Clear</button>
      </div>
    </div>
    <div class="flex flex-col rounded bg-neutral-600">
      <div class="flex flex-col items-stretch">
        <p class="label-head">Step</p>
        <p class="label">Current: {distance.toFixed(3)} mm</p>
      </div>
      <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1">
        <button class="btn-touch col-start-1 row-start-1" on:click={decreaseDistance}><Fa icon={faMinus} /></button>
        <button class="btn-touch col-start-2 row-start-1" on:click={increaseDistance}><Fa icon={faPlus} /></button>
      </div>
    </div>
  </div>
</div>
