<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa/src/fa.svelte';

  let nozzleTemp = moonraker.extruderCurrentTemperature;
  let bedTemp = moonraker.heaterBedCurrentTemperature;
  let nozzleTarget = moonraker.extruderTargetTemperature;
  let bedTarget = moonraker.heaterBedTargetTemperature;

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }
</script>

<div class="flex flex-row bg-neutral-800 p-2">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch bg-blue-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col">
    <div class="flex grow flex-wrap content-center items-center justify-center rounded">
      <div class="flex flex-row flex-wrap items-center gap-4">
        <div class="flex flex-col gap-2 rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600 ">
            <p class="label">Temperatures</p>
          </div>
          <div class="flex flex-col flex-wrap text-start ">
            <p class="label">Nozzle: {$nozzleTemp.toFixed(0)}/{$nozzleTarget.toFixed(0)} °C</p>
            <p class="label">Bed: {$bedTemp.toFixed(0)}/{$bedTarget.toFixed(0)} °C</p>
          </div>
        </div>
        <div class="flex flex-col gap-2 rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600 ">
            <p class="label">Fans</p>
          </div>
          <div class="flex flex-col flex-wrap text-start ">
            <p class="label">Nozzle: 100%</p>
            <p class="label">Extruder: 66%</p>
            <p class="label">Case: 100%</p>
          </div>
        </div>
        <div class="flex flex-col gap-2 rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-center rounded bg-red-600 ">
            <p class="label">Progress</p>
          </div>
          <div class="flex flex-col flex-wrap text-start ">
            <p class="label">ETA: 20:15</p>
            <p class="label">Layer: 20/4242</p>
            <p class="label">Printtime: 240min</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
