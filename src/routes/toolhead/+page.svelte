<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';

  let distance = 10;
  let extruderSpeed = 10;
  let moveSpeed = 25;

  let toolheadPosition = moonraker.toolheadPosition;

  async function home() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 0' // Home all untrusted axes
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function moveRelative(x: number = 0, y: number = 0, z: number = 0, e: number = 0) {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G0 X' + x + ' Y' + y + ' Z' + z + ' E' + e + ' F' + moveSpeed * 60
      }
    });
    await client.sendRequest(homeRequest);
  }

  async function extrudeRelative(e: number = 0) {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G0 E' + e + ' F' + extruderSpeed * 60
      }
    });
    await client.sendRequest(homeRequest);
  }
</script>

<div class="flex grow flex-row items-center justify-evenly gap-x-2 bg-neutral-900 text-neutral-200">
  <div class="grid grid-cols-4 grid-rows-4 items-stretch gap-2 rounded  bg-neutral-800 p-3 text-center">
    <div class="col-span-2 col-start-1 row-start-1 flex flex-col self-start py-2">
      <p>Move X, Y, Z</p>
      <p>Distance: {distance}</p>
    </div>
    <button class="btn-default x col-span-2 col-start-3 row-start-1 self-start" on:click={home}>Home</button>

    <button class="btn-default col-start-1 row-start-2 flex flex-col items-center justify-center" on:click={async () => moveRelative(-distance, distance, 0)}>
      <p>X-</p>
      <p>Y+</p>
    </button>
    <button class="btn-default col-start-2 row-start-2 flex aspect-square items-center justify-center" on:click={async () => moveRelative(0, distance, 0)}
      >Y+</button
    >
    <button
      class="btn-default col-start-3 row-start-2 flex aspect-square flex-col items-center justify-center"
      on:click={async () => moveRelative(distance, distance, 0)}
    >
      <p>X+</p>
      <p>Y+</p>
    </button>

    <button class="btn-default col-start-1 row-start-3 flex aspect-square items-center justify-center" on:click={async () => moveRelative(-distance, 0, 0)}
      >X-</button
    >
    <div class="col-start-2 row-start-3 flex aspect-square flex-col items-center justify-center">
      <p>X: {$toolheadPosition[0]}</p>
      <p>Y: {$toolheadPosition[1]}</p>
    </div>
    <button class="btn-default col-start-3 row-start-3 flex aspect-square items-center justify-center" on:click={async () => moveRelative(distance, 0, 0)}
      >X+</button
    >

    <button
      class="btn-default col-start-1 row-start-4 flex aspect-square flex-col items-center justify-center"
      on:click={async () => moveRelative(-distance, -distance, 0)}
    >
      <p>X-</p>
      <p>Y-</p>
    </button>
    <button class="btn-default col-start-2 row-start-4 flex aspect-square items-center justify-center" on:click={async () => moveRelative(0, -distance, 0)}
      >Y-</button
    >
    <button
      class="btn-default col-start-3 row-start-4 flex aspect-square flex-col items-center justify-center"
      on:click={async () => moveRelative(distance, -distance, 0)}
    >
      <p>X+</p>
      <p>Y-</p>
    </button>

    <button class="btn-default col-start-4 row-start-2 flex aspect-square items-center justify-center" on:click={async () => moveRelative(0, 0, distance)}
      >Z+</button
    >
    <div class="col-start-4 row-start-3  flex flex-col  items-center justify-center">
      <div>Z: {$toolheadPosition[2]}</div>
    </div>
    <button class="btn-default col-start-4 row-start-4 flex aspect-square items-center justify-center" on:click={async () => moveRelative(0, 0, -distance)}
      >Z-</button
    >
  </div>

  <div class="flex flex-col gap-1 rounded bg-neutral-800 p-3">
    <p class="self-start py-2">Distance</p>
    <button
      class={distance === 1 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 1;
      }}>1</button
    >
    <button
      class={distance === 2 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 2;
      }}>2</button
    >
    <button
      class={distance === 5 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 5;
      }}>5</button
    >
    <button
      class={distance === 10 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 10;
      }}>10</button
    >
    <button
      class={distance === 20 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 20;
      }}>20</button
    >
    <button
      class={distance === 50 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 50;
      }}>50</button
    >
    <button
      class={distance === 100 ? 'btn-primary' : 'btn-default'}
      on:click={() => {
        distance = 100;
      }}>100</button
    >
  </div>

  <div class="grid grid-cols-1 grid-rows-4 items-start gap-2 rounded bg-neutral-800 p-3 text-center">
    <p class="col-start-1 row-start-1 py-2">Filament</p>
    <button class="btn-default col-start-1 row-start-2 flex items-center  justify-center" on:click={async () => extrudeRelative(-distance)}>Retract</button>
    <div class="col-start-1 row-start-3 flex flex-col items-center justify-center ">
      <p>Distance: {distance}</p>
      <p>Speed: {extruderSpeed}</p>
    </div>
    <button class="btn-default col-start-1 row-start-4 flex items-center justify-center" on:click={async () => extrudeRelative(distance)}>Extrude</button>
  </div>
</div>
