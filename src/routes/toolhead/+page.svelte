<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { faArrowDown, faArrowLeft, faArrowRight, faArrowUp, faHome, faList, faMinus, faPlus, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let extruderSpeed = 5;
  let moveSpeed = 50;

  let toolheadPosition = moonraker.toolhead.Position;
  let nozzleTemp = moonraker.extruder.Temperature;

  let stepsArrIdx = 6;
  let stepsArr = [0.1, 0.2, 0.5, 1, 2, 5, 10, 20, 50, 100, 200];

  let distance = stepsArr[stepsArrIdx];
  let isHomedXY = false;
  let isHomedZ = false;

  moonraker.toolhead.HomedAxes.subscribe((value) => {
    if (value.includes('xy')) {
      isHomedXY = true;
    } else {
      isHomedXY = false;
    }
    if (value.includes('z')) {
      isHomedZ = true;
    } else {
      isHomedZ = false;
    }
  });

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

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
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
    stepsArrIdx = Math.min(stepsArrIdx + 1, stepsArr.length - 1);
    distance = stepsArr[stepsArrIdx];
  }

  function decreaseDistance() {
    stepsArrIdx = Math.max(stepsArrIdx - 1, 0);
    distance = stepsArr[stepsArrIdx];
  }
</script>

<div class="flex w-full flex-col flex-wrap items-stretch gap-2 bg-neutral-800 p-1">
  <div class="grid grid-cols-3 grid-rows-3 items-stretch justify-start gap-1 p-1">
    <button
      class="col-start-1 row-start-2 flex rounded border-l-4 border-yellow-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
      disabled={!isHomedXY}
      on:click={() => moveRelative(0, -distance, 0)}
    >
      <div class="self-center px-1"><Fa icon={faArrowLeft} /></div>
      <div class="flex-grow pl-2 pr-1 text-end">Left</div>
    </button>
    <button
      class="col-start-3 row-start-2 flex rounded border-l-4 border-yellow-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
      disabled={!isHomedXY}
      on:click={() => moveRelative(0, distance, 0)}
    >
      <div class="self-center px-1"><Fa icon={faArrowRight} /></div>
      <div class="flex-grow pl-2 pr-1 text-end">Right</div>
    </button>
    <button
      class="col-start-2 row-start-1 flex rounded border-l-4 border-yellow-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
      disabled={!isHomedXY}
      on:click={() => moveRelative(distance, 0, 0)}
    >
      <div class="self-center px-1"><Fa icon={faArrowUp} /></div>
      <div class="flex-grow pl-2 pr-1 text-end">Back</div>
    </button>
    <button
      class="col-start-2 row-start-3 flex rounded border-l-4 border-yellow-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500"
      disabled={!isHomedXY}
      on:click={() => moveRelative(-distance, 0, 0)}
    >
      <div class="self-center px-1"><Fa icon={faArrowDown} /></div>
      <div class="flex-grow pl-2 pr-1 text-end">Front</div>
    </button>
  </div>
</div>




<!--

<div class="flex flex-grow flex-row gap-1 bg-neutral-800 p-1">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch bg-red-600" on:click={() => goto('/parameter/zoffset')}>ZO</button>
    <div class="grow" />
    <button class="btn-touch bg-yellow-600" on:click={() => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col flex-wrap justify-around">
    <div class="flex flex-wrap content-center items-center justify-around">
      <div class="flex flex-col rounded bg-neutral-600">
        <div class="label-head flex flex-row flex-wrap justify-around">
          <p>Position</p>
          <p>X {$toolheadPosition[0].toFixed(1)}</p>
          <p>Y {$toolheadPosition[1].toFixed(1)}</p>
          <p>Z {$toolheadPosition[2].toFixed(1)}</p>
        </div>
        <div class="flex content-center items-center justify-center gap-2 p-1">
          <div class="grid grid-cols-3 grid-rows-3 gap-1">
            <button class="btn-touch col-start-1 row-start-1" on:click={homeXYZ}>
              <Fa icon={faHome} />
              <p>All</p>
            </button>
            <button class="btn-touch col-start-2 row-start-1" disabled={!isHomedXY} on:click={() => moveRelative(0, distance, 0)}>
              <p>Y</p>
              <Fa icon={faArrowUp} />
            </button>
            <button class="btn-touch col-start-1 row-start-2" disabled={!isHomedXY} on:click={() => moveRelative(-distance, 0, 0)}
              >X<Fa icon={faArrowLeft} />
            </button>
            <button class="btn-touch col-start-2 row-start-2" on:click={homeXY}>
              <Fa icon={faHome} />
              <p>XY</p>
            </button>
            <button class="btn-touch col-start-3 row-start-2" disabled={!isHomedXY} on:click={() => moveRelative(distance, 0, 0)}>
              <p>X</p>
              <Fa icon={faArrowRight} />
            </button>
            <button class="btn-touch col-start-2 row-start-3" disabled={!isHomedXY} on:click={() => moveRelative(0, -distance, 0)}
              >Y<Fa icon={faArrowDown} />
            </button>
          </div>
          <div class="grid grid-cols-1 grid-rows-3 gap-1">
            <button class="btn-touch col-start-1 row-start-1" disabled={!isHomedZ} on:click={() => moveRelative(0, 0, distance)}
              >Z<Fa icon={faArrowUp} /></button
            >
            <button class="btn-touch col-start-1 row-start-2" disabled={!isHomedXY} on:click={homeZ}>
              <Fa icon={faHome} />
              <p>Z</p>
            </button>
            <button class="btn-touch col-start-1 row-start-3" disabled={!isHomedZ} on:click={() => moveRelative(0, 0, -distance)}
              >Z<Fa icon={faArrowDown} /></button
            >
          </div>
        </div>
      </div>
      <div class="flex flex-col items-center gap-2">
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch">
            <p class="label-head">Extrude/Retract</p>
            <p class="label">Current: {$nozzleTemp.toFixed(0)} °C</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1">
            <button class="btn-touch col-start-1 row-start-1" on:click={() => extrudeRelative(-distance)}><Fa icon={faArrowDown} /></button>
            <button class="btn-touch col-start-2 row-start-1" on:click={() => extrudeRelative(distance)}><Fa icon={faArrowUp} /></button>
          </div>
        </div>
        <div class="flex flex-col rounded bg-neutral-600">
          <div class="items-stetch flex flex-col flex-wrap">
            <p class="label-head">Distance</p>
            <p class="label">Current: {distance} mm</p>
          </div>
          <div class="grid grid-cols-2 grid-rows-1 gap-1 p-1">
            <button class="btn-touch col-start-1 row-start-1" on:click={decreaseDistance}><Fa icon={faMinus} /></button>
            <button class="btn-touch col-start-2 row-start-1" on:click={increaseDistance}><Fa icon={faPlus} /></button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>



-->