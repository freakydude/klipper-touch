<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faPause, faPlay, faSkull, faStop } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let nozzleTemp = moonraker.extruderTemperature;
  let bedTemp = moonraker.heaterBedTemperature;
  let nozzleTarget = moonraker.extruderTarget;
  let bedTarget = moonraker.heaterBedTarget;
  let fanSpeed = moonraker.fanSpeed;
  let progress = moonraker.displayStatusProgress;
  let printState = moonraker.printStatsState;
  let printFilename = moonraker.printStatsFilename;
  // let klippyStateMessage = moonraker.klippyStateMessage;

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }

  async function resumePrint() {
    let resumeRequest = new JsonRpcRequest({
      method: 'printer.print.resume',
      params: {}
    });
    await client.sendRequest(resumeRequest);
  }

  async function pausePrint() {
    let pauseRequest = new JsonRpcRequest({
      method: 'printer.print.pause',
      params: {}
    });
    await client.sendRequest(pauseRequest);
  }

  async function cancelPrint() {
    let cancelRequest = new JsonRpcRequest({
      method: 'printer.print.cancel',
      params: {}
    });
    await client.sendRequest(cancelRequest);
  }
</script>

<div class="flex flex-grow flex-row gap-1 bg-neutral-800 p-1">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch bg-red-600" on:click={() => goto('/parameter/zoffset')}>ZO</button>
    <button class="btn-touch bg-red-600" on:click={() => goto('/files')}>FI</button>
    <div class="grow" />
    <button class="btn-touch bg-yellow-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col">
    <div class="flex grow flex-wrap content-center items-center justify-center rounded">
      <div class="flex flex-row flex-wrap items-center gap-4">
        <div class="flex flex-col gap-2 rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch ">
            <p class="label-head">Status</p>
            <p class="label">Nozzle: {$nozzleTemp.toFixed(0)}/{$nozzleTarget.toFixed(0)} °C</p>
            <p class="label">Bed: {$bedTemp.toFixed(0)}/{$bedTarget.toFixed(0)} °C</p>
            <p class="label">Fan: {($fanSpeed * 100).toFixed(0)} %</p>
          </div>
        </div>
        <div class="flex flex-col gap-2 rounded bg-neutral-600">
          <div class="flex flex-col flex-wrap items-stretch ">
            <p class="label-head ">Print</p>
            <p class="label">State: {$printState}</p>
            <p class="label">File: {$printFilename}</p>
            <p class="label">Progress: {($progress * 100).toFixed(1)} %</p>
            <div class="grid grid-cols-2 grid-rows-1  gap-1 p-1 ">
              {#if $printState == 'paused'}
                <button class="btn-touch col-start-1 p-4" on:click={resumePrint}>
                  <Fa icon={faPlay} />
                </button>
              {:else if $printState == 'printing'}
                <button class="btn-touch col-start-1 p-4" on:click={pausePrint}>
                  <Fa icon={faPause} />
                </button>
              {/if}
              {#if $printState == ('paused' || 'printing')}
                <button class="btn-touch col-start-2 p-4" on:click={cancelPrint}>
                  <Fa icon={faStop} />
                </button>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
