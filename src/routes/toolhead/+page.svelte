<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let distance = 10;
  let extruderSpeed = 5;
  let moveSpeed = 50;

  let toolheadPosition = moonraker.toolheadPosition;
  let nozzleTemp = moonraker.extruderTemperature;

  async function home() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28' // Home all untrusted axes
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

  async function extrudeRelative(e: number = 0) {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G91\n G1 E' + e + ' F' + extruderSpeed * 60
      }
    });
    await client.sendRequest(homeRequest);
  }
</script>

<div class="flex grow flex-wrap items-center justify-evenly gap-2 bg-neutral-800">
  <div class="flex flex-col flex-wrap gap-1 gap-y-8 ">
    <button class="btn-touch bg-red-900" on:click={() => goto('/')}>Menu</button>
    <button class="btn-touch" on:click={home}>Home XYZ</button>
  </div>
  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => moveRelative(distance, 0, 0)}>X +</button>
    <p class="label">{$toolheadPosition[0].toFixed(0)}</p>
    <button class="btn-touch" on:click={async () => moveRelative(-distance, 0, 0)}>X -</button>
  </div>
  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => moveRelative(0, distance, 0)}>Y +</button>
    <p class="label">{$toolheadPosition[1].toFixed(0)}</p>
    <button class="btn-touch" on:click={async () => moveRelative(0, -distance, 0)}>Y -</button>
  </div>
  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => moveRelative(0, 0, distance)}>Z +</button>
    <p class="label">{$toolheadPosition[2].toFixed(0)}</p>
    <button class="btn-touch" on:click={async () => moveRelative(0, 0, -distance)}>Z -</button>
  </div>

  <div class="flex flex-col flex-wrap gap-1 bg-red-900">
    <button class="btn-touch" on:click={async () => extrudeRelative(-distance)}>Retract</button>
    <p class="label">{$nozzleTemp.toFixed(0)} °C</p>
    <button class="btn-touch" on:click={async () => extrudeRelative(distance)}>Extrude</button>
  </div>

  <div class="flex flex-row flex-wrap items-center gap-1 bg-red-900 ">
    <button
      class="btn-touch h-16 w-20 {extruderSpeed === 1 ? 'selected' : ''}"
      on:click={() => {
        extruderSpeed = 1;
      }}>1</button
    >
    <button
      class="btn-touch h-16 w-20 {extruderSpeed === 2 ? 'selected' : ''}"
      on:click={() => {
        extruderSpeed = 2;
      }}>2</button
    >
    <p class="label w-32">E-Speed</p>
    <button
      class="btn-touch h-16 w-20 {extruderSpeed === 5 ? 'selected' : ''}"
      on:click={() => {
        extruderSpeed = 5;
      }}>5</button
    >
    <button
      class="btn-touch h-16 w-20 {extruderSpeed === 10 ? 'selected' : ''}"
      on:click={() => {
        extruderSpeed = 10;
      }}>10</button
    >
  </div>
  <div class="flex flex-row flex-wrap items-center gap-1 bg-red-900">
    <button
      class="btn-touch h-16 w-20 {distance === 1 ? 'selected' : ''}"
      on:click={() => {
        distance = 1;
      }}>1</button
    >
    <button
      class="btn-touch h-16 w-20 {distance === 2 ? 'selected' : ''}"
      on:click={() => {
        distance = 2;
      }}>2</button
    >
    <button
      class="btn-touch h-16 w-20 {distance === 5 ? 'selected' : ''}"
      on:click={() => {
        distance = 5;
      }}>5</button
    >
    <p class="label w-32">Distance</p>
    <button
      class="btn-touch h-16 w-20 {distance === 10 ? 'selected' : ''}"
      on:click={() => {
        distance = 10;
      }}>10</button
    >
    <button
      class="btn-touch h-16 w-20 {distance === 20 ? 'selected' : ''}"
      on:click={() => {
        distance = 20;
      }}>20</button
    >
    <button
      class="btn-touch h-16 w-20 {distance === 50 ? 'selected' : ''}"
      on:click={() => {
        distance = 50;
      }}>50</button
    >
  </div>
</div>
