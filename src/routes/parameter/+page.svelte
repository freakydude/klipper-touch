<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let nozzleTemp = moonraker.extruderTemperature;
  let bedTemp = moonraker.heaterBedTemperature;
  let zOffset = moonraker.gcodeZOffset;
  let tempSteps = 10;
  let zOffsetTarget = 0.0;
  let nozzleTempTarget = 0.0;
  let bedTempTarget = 0.0;
  let offsetSteps = 0.02;

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
