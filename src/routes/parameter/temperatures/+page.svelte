<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import Fa from 'svelte-fa/src/fa.svelte';
  import { faArrowDown, faArrowUp, faList, faMinus, faPlus, faSkull } from '@fortawesome/free-solid-svg-icons';

  let stepsArrIdx = 4;
  let stepsArr = [1, 2, 5, 10, 20, 50, 100];

  let nozzleTemp = moonraker.extruderCurrentTemperature;
  let nozzleTempTarget = moonraker.extruderTargetTemperature;

  let bedTemp = moonraker.heaterBedCurrentTemperature;
  let bedTempTarget = moonraker.heaterBedTargetTemperature;

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

<div class="flex flex-row bg-neutral-800 p-1 gap-1">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/parameter')}>PA</button>
    <button class="btn-touch flex flex-col bg-yellow-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col">
    <div class="flex grow flex-wrap content-center items-center justify-around rounded">
      <div class="flex flex-row flex-wrap items-center gap-4">
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch">
            <p class="label-head">Bed</p>
            <p class="label">Current: {$bedTemp.toFixed(0)} °C</p>
            <p class="label">Target: {$bedTempTarget.toFixed(0)} °C</p>
          </div>
          <div class="grid grid-cols-1 grid-rows-2 gap-1 p-1">
            <button class="btn-touch col-start-1 row-start-1 " on:click={async () => changeBedTemperature(distance)}><Fa icon={faArrowUp} /></button>
            <button class="btn-touch col-start-1 row-start-2 " on:click={async () => changeBedTemperature(-distance)}><Fa icon={faArrowDown} /></button>
          </div>
        </div>
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch">
            <p class="label-head">Nozzle</p>
            <p class="label">Current: {$nozzleTemp.toFixed(0)} °C</p>
            <p class="label">Target: {$nozzleTempTarget.toFixed(0)} °C</p>
          </div>
          <div class="grid grid-cols-1 grid-rows-2 gap-1 p-1">
            <button class="btn-touch col-start-1 row-start-1 " on:click={async () => changeNozzleTemperature(distance)}><Fa icon={faArrowUp} /></button>
            <button class="btn-touch col-start-1 row-start-2 " on:click={async () => changeNozzleTemperature(-distance)}><Fa icon={faArrowDown} /></button>
          </div>
        </div>
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch">
            <p class="label-head">Steps</p>
            <p class="label">Current: {distance.toFixed(0)} °C</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1 ">
            <button class="btn-touch col-start-1 row-start-1" on:click={decreaseDistance}><Fa icon={faMinus} /></button>
            <button class="btn-touch col-start-2 row-start-1" on:click={increaseDistance}><Fa icon={faPlus} /></button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
