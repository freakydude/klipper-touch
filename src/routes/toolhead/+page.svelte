<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let distance = 10;
  let extruderSpeed = 5;
  let moveSpeed = 50;

  let toolheadPosition = moonraker.toolheadPosition;
  let nozzleTemp = moonraker.extruderTemperature;

  async function homeXY() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 X Y' // Home all untrusted axes
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function homeZ() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 Z' // Home all untrusted axes
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

<div class="flex flex-row  bg-green-500">
  <div class="  flex flex-col  bg-blue-800 ">
    <button class="btn-touch flex flex-col" on:click={() => goto('/')}>
      <i class="fa-solid fa-bars" />
    </button>
    <p class="label">teil2</p>
  </div>

  <div class="flex flex-col bg-yellow-500 ">
    <div class="flex flex-row bg-red-500 ">
      <p class="label">Test</p>
      <p class="label">Test</p>
      <p class="label">Test</p>
      <p class="label">Test</p>
      <p class="label">Test</p>
    </div>
    <div class="flex content-center justify-around justify-self-stretch bg-pink-500">
      <div class="grid grid-cols-3 grid-rows-3 gap-1 bg-slate-300">
        <button class="btn-touch col-start-2 row-start-1" on:click={async () => moveRelative(0, distance, 0)}>Y +</button>
        <button class="btn-touch col-start-1 row-start-2" on:click={async () => moveRelative(-distance, 0, 0)}>X -</button>
        <button class="btn-touch col-start-2 row-start-2" on:click={homeXY}><i class="fa-solid fa-house" /></button>
        <button class="btn-touch col-start-3 row-start-2" on:click={async () => moveRelative(distance, 0, 0)}>X +</button>
        <button class="btn-touch col-start-2 row-start-3" on:click={async () => moveRelative(0, -distance, 0)}>Y -</button>
      </div>
      <div class="grid grid-cols-1 grid-rows-3 gap-1 bg-slate-500">
        <button class="btn-touch col-start-1 row-start-1" on:click={async () => moveRelative(0, 0, distance)}>Z +</button>
        <button class="btn-touch col-start-1 row-start-2" on:click={homeZ}><i class="fa-solid fa-house" /></button>
        <button class="btn-touch col-start-1 row-start-3" on:click={async () => moveRelative(0, 0, -distance)}>Z -</button>
      </div>

      <div class="grid grid-cols-1 grid-rows-2 gap-1 bg-slate-700">
        <button class="btn-touch col-start-1 row-start-1" on:click={async () => extrudeRelative(-distance)}>E +</button>
        <button class="btn-touch col-start-1 row-start-2" on:click={async () => extrudeRelative(distance)}>E -</button>
      </div>

      <!-- <div class="flex flex-col flex-wrap ">
    </div>
    <div class="flex flex-col flex-wrap bg-red-900">      
      <p class="label">{$toolheadPosition[0].toFixed(0)}</p>
    </div>
    <div class="flex flex-col flex-wrap  bg-red-900">
      
      <p class="label">{$toolheadPosition[1].toFixed(0)}</p>
      
    </div>
    <div class="flex flex-col flex-wrap  bg-red-900">
      <p class="label">{$toolheadPosition[2].toFixed(0)}</p>
    </div>

    <div class="flex flex-col flex-wrap  bg-red-900">
      <p class="label">{$nozzleTemp.toFixed(0)} °C</p>
    </div>

    <div class="flex flex-row flex-wrap items-center bg-red-900 ">
      <p class="label">E-Speed</p>
      <button
        class="btn-touch  {extruderSpeed === 1 ? 'selected' : ''}"
        on:click={() => {
          extruderSpeed = 1;
        }}>1</button
      >
      <button
        class="btn-touch {extruderSpeed === 2 ? 'selected' : ''}"
        on:click={() => {
          extruderSpeed = 2;
        }}>2</button
      >

      <button
        class="btn-touch  {extruderSpeed === 5 ? 'selected' : ''}"
        on:click={() => {
          extruderSpeed = 5;
        }}>5</button
      >
      <button
        class="btn-touch  {extruderSpeed === 10 ? 'selected' : ''}"
        on:click={() => {
          extruderSpeed = 10;
        }}>10</button
      >
    </div>
    <div class="flex flex-row flex-wrap items-center gap-1 bg-red-900">
      <p class="label ">Distance</p>
      <button
        class="btn-touch {distance === 1 ? 'selected' : ''}"
        on:click={() => {
          distance = 1;
        }}>1</button
      >
      <button
        class="btn-touch  {distance === 2 ? 'selected' : ''}"
        on:click={() => {
          distance = 2;
        }}>2</button
      >
      <button
        class="btn-touch {distance === 5 ? 'selected' : ''}"
        on:click={() => {
          distance = 5;
        }}>5</button
      >

      <button
        class="btn-touch  {distance === 10 ? 'selected' : ''}"
        on:click={() => {
          distance = 10;
        }}>10</button
      >
      <button
        class="btn-touch  {distance === 20 ? 'selected' : ''}"
        on:click={() => {
          distance = 20;
        }}>20</button
      >
      <button
        class="btn-touch  {distance === 50 ? 'selected' : ''}"
        on:click={() => {
          distance = 50;
        }}>50</button
      >
    </div> -->
    </div>
  </div>
</div>
