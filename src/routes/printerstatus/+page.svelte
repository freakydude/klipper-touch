<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

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

<div class="flex w-full flex-col items-stretch justify-between   gap-2 bg-neutral-800 p-2">
  <div class="flex flex-row flex-wrap items-center justify-start gap-1 p-1">
    <button class="flex rounded border-l-4 border-blue-400 bg-neutral-700 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Nozzle</div>
      <div class="flex-grow pl-2 pr-1 text-end">180 / 210 °C</div>
    </button>
    <button class="flex rounded border-l-4 border-red-400 bg-neutral-700 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Bed</div>
      <div class="flex-grow pl-2 pr-1 text-end">70 / 60 °C</div>
    </button>
  </div>
  <div class="flex flex-row flex-wrap items-center justify-start gap-1 p-1">
    <button class="flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Fan</div>
      <div class="flex-grow pl-2 pr-1 text-end">75 %</div>
    </button>
    <button class=" flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Speed</div>
      <div class="flex-grow pl-2 pr-1 text-end">100 %</div>
    </button>
    <button class="flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Flow</div>
      <div class="flex-grow pl-2 pr-1 text-end">98 %</div>
    </button>
  </div>
  <div class="flex flex-row flex-wrap items-center justify-start gap-1 p-1">
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Layer</div>
      <div class="flex-grow pl-2 pr-1 text-end">10 / 2300</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Height</div>
      <div class="flex-grow pl-2 pr-1 text-end">40.2 mm</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Printed</div>
      <div class="flex-grow pl-2 pr-1 text-end">1 h 13 min</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Estimation</div>
      <div class="flex-grow pl-2 pr-1 text-end">2 h 46 min</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">ETA</div>
      <div class="flex-grow pl-2 pr-1 text-end">20:45</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Progress</div>
      <div class="flex-grow pl-2 pr-1 text-end">60 %</div>
    </div>
    <div class="flex rounded bg-neutral-700 px-1 py-2 text-white shadow">
      <div class="px-1">Filament</div>
      <div class="flex-grow pl-2 pr-1 text-end">30 / 91 m</div>
    </div>
  </div>
  <div class="flex flex-row flex-wrap items-center justify-start gap-1 p-1">
    <button class="flex rounded border-l-4 border-yellow-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Pause</div>
    </button>
    <button class="flex rounded border-l-4 border-purple-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">Machine</div>
    </button>
    <button class="flex rounded border-l-4 border-red-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500">
      <div class="px-1">EM</div>
    </button>
  </div>
</div>
