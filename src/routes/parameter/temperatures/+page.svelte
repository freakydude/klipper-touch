<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let nozzleTemp = moonraker.extruderTemperature;
  let bedTemp = moonraker.heaterBedTemperature;
  let tempSteps = 10;
  let nozzleTempTarget = 0.0;
  let bedTempTarget = 0.0;

  async function changeNozzleTemperature(tempSteps: number) {
    nozzleTempTarget = Math.max(0, nozzleTempTarget + tempSteps);

    let nozzleTempRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M104 S' + nozzleTempTarget
      }
    });
    await client.sendRequest(nozzleTempRequest);
  }

  async function changeBedTemperature(tempSteps: number) {
    bedTempTarget = Math.max(0, bedTempTarget + tempSteps);

    let bedTempRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M140 S' + bedTempTarget
      }
    });
    await client.sendRequest(bedTempRequest);
  }
</script>

<div class="flex grow flex-wrap items-center justify-evenly gap-2 bg-neutral-800">
  <div class="flex flex-col flex-wrap gap-1">
    <button class="btn-touch bg-red-900" on:click={() => goto('/')}>Menu</button>
  </div>

  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => changeNozzleTemperature(tempSteps)}>{(nozzleTempTarget + tempSteps).toFixed(0)} °C</button>
    <p class="label">Nozzle</p>
    <p class="label">{$nozzleTemp.toFixed(0)} °C</p>
    <button class="btn-touch" on:click={async () => changeNozzleTemperature(-tempSteps)}>{(nozzleTempTarget - tempSteps).toFixed(0)} °C</button>
  </div>

  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => changeBedTemperature(tempSteps)}>{(bedTempTarget + tempSteps).toFixed(0)} °C</button>
    <p class="label">Bed</p>
    <p class="label">{$bedTemp.toFixed(0)} °C</p>
    <button class="btn-touch" on:click={async () => changeBedTemperature(-tempSteps)}>{(bedTempTarget - tempSteps).toFixed(0)} °C</button>
  </div>

  <div class="flex flex-row flex-wrap items-center gap-1 bg-red-900">
    <button
      class="btn-touch h-16 w-20 {tempSteps === 1 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 1;
      }}>1</button
    >
    <button
      class="btn-touch h-16 w-20 {tempSteps === 2 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 2;
      }}>2</button
    >
    <button
      class="btn-touch h-16 w-20 {tempSteps === 5 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 5;
      }}>5</button
    >
    <p class="label w-32">T-Steps</p>
    <button
      class="btn-touch h-16 w-20 {tempSteps === 10 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 10;
      }}>10</button
    >
    <button
      class="btn-touch h-16 w-20 {tempSteps === 20 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 20;
      }}>20</button
    >
    <button
      class="btn-touch h-16 w-20 {tempSteps === 50 ? 'selected' : ''}"
      on:click={() => {
        tempSteps = 50;
      }}>50</button
    >
  </div>
</div>
