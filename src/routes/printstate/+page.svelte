<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { faSquareFull } from '@fortawesome/free-regular-svg-icons';
  import { faPause, faStop, faSkull, faGear, faDownLong, faArrowsUpDown, faArrowsUpDownLeftRight } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  let toolheadPosition = moonraker.toolhead.Position;
  let nozzleTemp = moonraker.extruder.Temperature;
  let bedTemp = moonraker.heaterBed.Temperature;
  let nozzleTarget = moonraker.extruder.Target;
  let bedTarget = moonraker.heaterBed.Target;
  let fanSpeed = moonraker.fan.Speed;
  let progress = moonraker.displayStatus.Progress;
  let printState = moonraker.printStats.State;
  let printStatsMessage = moonraker.printStats.Message;
  let printStatsPrintDuration = moonraker.printStats.PrintDuration;
  let printFilename = moonraker.printStats.Filename;
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
  <p class="fixed left-2 top-1 text-sm text-neutral-100">Ready</p>

  <div class="flex flex-grow flex-row justify-between gap-3">
    <div class="flex flex-col items-end justify-center gap-3 p-2 pl-2 pr-1">
      <div class="flex flex-col items-end gap-1 rounded-lg bg-neutral-500 px-3 py-2">
        <table class="table-auto text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">Nozzle</td>
            <td class="text-end">130</td>
            <td>/</td>
            <td class="text-end">210</td>
            <td>°C</td>
          </tr>
          <tr>
            <td class="pr-2 text-end">Bed</td>
            <td class="text-end">101</td>
            <td>/</td>
            <td class="text-end">115 </td>
            <td>°C</td>
          </tr>
        </table>
      </div>
      <div class="flex flex-col items-end gap-1 rounded-lg bg-neutral-500 px-3 py-2">
        <table class="table-auto text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">X</td>
            <td class="text-end">255.8</td>
            <td>mm</td>
          </tr>
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">Y</td>
            <td class="text-end">142.3</td>
            <td>mm</td>
          </tr>
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">Z</td>
            <td class="text-end">13.1</td>
            <td>mm</td>
          </tr>
          <tr>
            <td class="pr-2 text-end">Z-Off</td>
            <td class="text-end">0.133</td>
            <td>mm</td>
          </tr>
        </table>
      </div>
    </div>
    <div class="flex flex-col items-center justify-center gap-3 px-1 py-2">
      <div class="flex flex-col items-center gap-1 rounded-lg bg-neutral-500 px-3 py-2">
        <p class="w-40 overflow-clip text-center text-sm text-neutral-50">Benchy3d-PLA-F1.0-S80</p>
        <div class="flex h-32 w-32 flex-col items-stretch justify-center rounded-lg bg-orange-400">
          <p class="text-center">Loaded Print</p>
          <p class="text-center">Preview Picture</p>
          <p class="text-center">here</p>
        </div>
        <table class="table-auto text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">Estimate:</td>
            <td>3h15 / 20:47</td>
          </tr>
          <tr>
            <td class="pr-2 text-end">Weight:</td>
            <td>105 g</td>
          </tr>
        </table>
      </div>
    </div>
    <div class="flex flex-col items-end justify-center gap-3 py-2 pl-1">
      <button
        class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
        Load
      </button>
      <button
        class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
        Start
      </button>
    </div>
  </div>

  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-400 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
      State
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
      Move
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
      Temp
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
      Baby
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-400">
      Prep
    </button>
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">15:32</p>
    </div>
    <button class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-red-800 drop-shadow-md hover:bg-neutral-400">
      Kill
    </button>
  </div>
</div>
