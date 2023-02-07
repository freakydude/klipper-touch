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
    distance += 10;
  }

  function decreaseDistance() {
    distance -= 10;
  }
</script>

<div class="flex flex-row bg-neutral-700 p-2">
  <div class="flex flex-col justify-between   ">
    <button class="btn-touch fa-solid fa-list flex flex-col bg-red-600  " on:click={() => goto('/')} />
  </div>

  <div class="flex grow flex-col">
    <div class="flex grow flex-wrap content-center items-center justify-around  rounded">
      <div class="flex flex-col">
        <div class="flex flex-row flex-wrap justify-around rounded bg-red-600">
          <p class="label">X {$toolheadPosition[0].toFixed(1)}</p>
          <p class="label">Y {$toolheadPosition[1].toFixed(1)}</p>
          <p class="label">Z {$toolheadPosition[2].toFixed(1)}</p>
        </div>
        <div class="flex content-center items-center justify-center gap-2 rounded bg-neutral-600 p-1">
          <div class="grid grid-cols-3 grid-rows-3 gap-1">
            <button class="btn-touch fa-solid fa-home col-start-1 row-start-1" on:click={homeXYZ} />
            <button class="btn-touch col-start-2 row-start-1" on:click={async () => moveRelative(0, distance, 0)}>Y <i class="fa-solid fa-arrow-up" /></button>
            <button class="btn-touch col-start-1 row-start-2" on:click={async () => moveRelative(-distance, 0, 0)}
              >X <i class="fa-solid fa-arrow-left" /></button>
            <button class="btn-touch fa-solid fa-arrows-up-down-left-right col-start-2 row-start-2" on:click={homeXY} />
            <button class="btn-touch col-start-3 row-start-2" on:click={async () => moveRelative(distance, 0, 0)}
              >X <i class="fa-solid fa-arrow-right" /></button>
            <button class="btn-touch col-start-2 row-start-3" on:click={async () => moveRelative(0, -distance, 0)}
              >Y <i class="fa-solid fa-arrow-down" /></button>
          </div>
          <div class="grid grid-cols-1 grid-rows-3 gap-1">
            <button class="btn-touch col-start-1 row-start-1" on:click={async () => moveRelative(0, 0, distance)}>Z <i class="fa-solid fa-arrow-up" /></button>
            <button class="btn-touch fa-solid fa-arrows-up-down col-start-1 row-start-2" on:click={homeZ} />
            <button class="btn-touch col-start-1 row-start-3" on:click={async () => moveRelative(0, 0, -distance)}
              >Z <i class="fa-solid fa-arrow-down" /></button>
          </div>
        </div>
      </div>
      <div class="flex flex-col gap-4">
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600">
            <p class="label">Extrude {$nozzleTemp.toFixed(0)} °C</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1">
            <button class="btn-touch fa-solid fa-arrow-down col-start-1 row-start-1 " on:click={async () => extrudeRelative(-distance)} />
            <button class="btn-touch fa-solid fa-arrow-up col-start-2 row-start-1 " on:click={async () => extrudeRelative(distance)} />
          </div>
        </div>
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600">
            <p class="label">Distance {distance} mm</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1">
            <button class="btn-touch fa-solid fa-plus col-start-1 row-start-1" on:click={increaseDistance} />
            <button class="btn-touch fa-solid fa-minus col-start-2 row-start-1" on:click={decreaseDistance} />
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
