<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { writable } from 'svelte/store';
  import type { TPrintState } from '$lib/moonraker/types/TPrintState';

  let maxAcceleration = moonraker.toolhead.MaxAcceleration;
  let toolheadPosition = moonraker.toolhead.Position;
  let nozzleTemp = moonraker.extruder.Temperature;
  let bedTemp = moonraker.heaterBed.Temperature;
  let nozzleTarget = moonraker.extruder.Target;
  let pressureAdvance = moonraker.extruder.PressureAdvance;
  let bedTarget = moonraker.heaterBed.Target;
  let fanSpeed = moonraker.fan.Speed;
  let baby = moonraker.gcodeMove.HomeOrigin;
  let speed = moonraker.gcodeMove.Speed;
  let speedFactor = moonraker.gcodeMove.SpeedFactor;
  let extrudeFactor = moonraker.gcodeMove.ExtrudeFactor;

  let printState = moonraker.printStats.State;
  // let printState = writable<TPrintState>('printing');
  let printStatsMessage = moonraker.printStats.Message;
  let printFilename = moonraker.printStats.Filename;
  let printStatsPrintDuration = moonraker.printStats.PrintDuration;
  let filamentUsed = moonraker.printStats.FilamentUsed;
  let currentLayer = moonraker.printStats.Info.CurrentLayer;
  let totalLayer = moonraker.printStats.Info.TotalLayer;

  let progress = moonraker.displayStatus.Progress;

  let fileEstimatedPrintTime = 0.0;
  let remainingPrintingTime = 0.0;

  // let hours = new Date(Date.now() / 100.0 + remainingPrintingTime).getHours();
  // let minutes = new Date(Date.now() / 100.0 + remainingPrintingTime).getMinutes();

  printStatsPrintDuration.subscribe((value) => {
    remainingPrintingTime = Math.max(0, fileEstimatedPrintTime - value);
  });

  printFilename.subscribe(async (value) => {
    if (value !== '') {
      try {
        fileEstimatedPrintTime = await getEstimatedPrintTime(value + '.gcode');
      } catch (error) {
        console.log(error);
      }
    }
  });

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

  async function getEstimatedPrintTime(fileName: string): Promise<number> {
    let fileMetaRequest = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: fileName
      }
    });

    console.log(fileName);
    let fileMetaResponse = await client.sendRequest(fileMetaRequest);

    let fileMeta = fileMetaResponse.result;

    return fileMeta.estimated_time;
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex flex-grow flex-row justify-between gap-3">
    <span class="flex flex-grow flex-row justify-around p-1">
      <div class="flex flex-col items-end justify-center gap-3 py-2 pr-1">
        <div class="flex flex-col items-stretch gap-1 rounded-lg bg-neutral-600 px-3 py-2">
          <table class="table-auto text-sm text-neutral-50">
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="pr-1 text-end">{$nozzleTemp.toFixed(1)} / {$nozzleTarget}</td>
              <td class="text-start">°C</td>
            </tr>
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Bed</td>
              <td class="pr-1 text-end">{$bedTemp.toFixed(1)} / {$bedTarget}</td>
              <td class="text-start">°C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Fan</td>
              <td class="pr-1 text-end">{($fanSpeed * 100.0).toFixed(1)}</td>
              <td class="text-start">%</td>
            </tr>
            {#if $printState === 'printing'}
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="pr-1 text-end">{$speed.toFixed(0)}</td>
                <td class="text-start">mm/s</td>
              </tr>
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Flow</td>
                <td class="pr-1 text-end">0.00</td>
                <td class="text-start">mm³/s</td>
              </tr>
            {/if}
          </table>
        </div>
        <div class="flex flex-col items-stretch gap-1 rounded-lg bg-neutral-600 px-3 py-2">
          <table class="table-auto text-sm text-neutral-50">
            {#if $printState === 'standby' || $printState === 'cancelled' || $printState === 'complete' || $printState === 'error'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">X</td>
                <td class="pr-1 text-end">{$toolheadPosition[0].toFixed(2)}</td>
                <td class="text-start">mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Y</td>
                <td class="pr-1 text-end">{$toolheadPosition[1].toFixed(2)}</td>
                <td class="text-start">mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Z</td>
                <td class="pr-1 text-end">{$toolheadPosition[2].toFixed(2)}</td>
                <td class="text-start">mm</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Baby</td>
                <td class="pr-1 text-end">{$baby.toFixed(3)}</td>
                <td class="text-start">mm</td>
              </tr>
            {:else if $printState === 'printing' || $printState === 'paused'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="pr-1 text-end">{($speedFactor * 100.0).toFixed(0)}</td>
                <td class="text-start">%</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Extrude</td>
                <td class="pr-1 text-end">{($extrudeFactor * 100.0).toFixed(0)}</td>
                <td class="text-start">%</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Accel</td>
                <td class="pr-1 text-end">{$maxAcceleration.toFixed(0)}</td>
                <td class="text-start">mm/s²</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Pressure</td>
                <td class="pr-1 text-end">{$pressureAdvance.toFixed(3)}</td>
                <td class="text-start">x</td>
              </tr>
            {/if}
          </table>
        </div>
      </div>
      <div class="flex flex-col items-center justify-center gap-3 py-2 pl-1">
        <div class="flex flex-col items-center gap-1 rounded-lg bg-neutral-600 px-3 py-2">
          {#if $printState === 'standby' || $printState === 'cancelled' || $printState === 'complete' || $printState === 'printing' || $printState === 'paused' || $printState === 'error'}
            <p class="overflow-hidden text-center text-sm text-neutral-50">{$printFilename.replace('.gcode', '').substring(0, 20)}</p>
            <div class="flex h-36 w-36 flex-col items-stretch justify-center rounded-lg bg-orange-400">
              <p class="text-center">Loaded Print</p>
              <p class="text-center">Preview Picture</p>
              <p class="text-center">here</p>
            </div>
          {/if}

          <table class="table-auto text-sm text-neutral-50">
            {#if $printState === 'standby' || $printState === 'cancelled' || $printState === 'complete' || $printState === 'error'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">ETA</td>
                <td>3h15 - 20:47</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Weight</td>
                <td>105 g</td>
              </tr>
            {:else if $printState === 'printing' || $printState === 'paused'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Progress</td>
                <td>{($progress * 100.0).toFixed(1)}</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Layer</td>
                <td>{$currentLayer} / {$totalLayer}</td>
              </tr>
              <tr class="">
                <td class="pr-2 text-end">Filament</td>
                <td>{($filamentUsed / 100.0).toFixed(2)} m</td>
              </tr>
            {/if}
          </table>
        </div>
      </div>
    </span>
    <div class="flex flex-col items-end justify-center gap-3 py-2">
      {#if $printState === 'standby' || $printState === 'cancelled' || $printState === 'complete'}
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Load
        </button>
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Start
        </button>
      {:else if $printState === 'printing'}
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Pause
        </button>
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Cancel
        </button>
      {:else if $printState === 'paused'}
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Continue
        </button>
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Cancel
        </button>
      {:else if $printState === 'error'}
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-red-800 drop-shadow-md hover:bg-neutral-500">
          Error
        </button>
      {/if}
    </div>
  </div>

  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
      State
    </button>
    {#if $printState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
        Move
      </button>
    {/if}
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
      Temp
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
      Baby
    </button>
    {#if $printState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
        Prep
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">15:32</p>
    </div>
    <button class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-800 drop-shadow-md hover:bg-neutral-500">
      Kill
    </button>
  </div>
</div>
