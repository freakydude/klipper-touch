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

<div class="flex w-full flex-row border-t border-green-400 bg-neutral-800">
  <div class="flex w-1/4 flex-col items-stretch justify-around p-1">
    <span class="flex flex-col justify-center gap-1">
      <button class="flex rounded border-l-4 border-red-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => goto('/temp_nozzle')}>
        <div class="self-center px-1"><Fa icon={faDownLong} /></div>
        <div class="flex-grow pr-1 text-end">{$nozzleTemp.toFixed(1)}/{$nozzleTarget.toFixed(0)} °C</div>
      </button>
      <button class="flex rounded border-l-4 border-blue-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => goto('/temp_bed')}>
        <div class="self-center px-1"><Fa icon={faSquareFull} /></div>
        <div class="flex-grow text-end">{$bedTemp.toFixed(1)}/{$bedTarget.toFixed(0)} °C</div>
      </button>
    </span>
    <span class="flex flex-col justify-center gap-1">
      <button class="flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500">
        <div class="self-center px-1">Fan 0</div>
        <div class="flex-grow text-end">{$fanSpeed * 100} %</div>
      </button>
      <button class="flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500">
        <div class="self-center px-1">Speed</div>
        <div class="flex-grow text-end">100 %</div>
      </button>
      <button class="flex rounded border-l-4 border-gray-400 bg-neutral-700 px-1 py-2 text-white hover:bg-neutral-500">
        <div class="self-center px-1">Flow</div>
        <div class="flex-grow text-end">98 %</div>
      </button>
    </span>
  </div>
  <div class="flex w-2/4 flex-col items-stretch justify-around gap-1 p-1">
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Layer</div>
      <div class="flex-grow px-1 text-end">10 / 2300</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Z</div>
      <div class="flex-grow px-1 text-end">{$toolheadPosition[2].toFixed(2)} mm</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Filament</div>
      <div class="flex-grow px-1 text-end">67.4 m</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Printed</div>
      <div class="flex-grow px-1 text-end">{$printStatsPrintDuration.toFixed(1)}</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Estimation</div>
      <div class="flex-grow px-1 text-end">2h 33m</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">ETA</div>
      <div class="flex-grow px-1 text-end">20:45</div>
    </div>
    <div class="flex border-b border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Flow</div>
      <div class="flex-grow px-1 text-end">60 mm³/s</div>
    </div>
    <div class="flex border-neutral-600 px-1 py-1 text-white">
      <div class="px-1 text-green-400">Speed</div>
      <div class="flex-grow px-1 text-end">80 mm/s</div>
    </div>
  </div>
  <div class="flex w-1/4 flex-col items-stretch justify-around p-1">
    <span class="flex flex-col justify-center gap-1">
      <button class="flex rounded border-l-4 border-yellow-400 bg-neutral-600 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => pausePrint()}>
        <div class="self-center px-1"><Fa icon={faPause} /></div>
        <div class="flex-grow pr-1 text-end">Pause</div>
      </button>
      <button
        class="flex rounded border-l-4 border-orange-400 bg-neutral-600 px-1 py-2 text-white hover:bg-neutral-500 disabled:text-neutral-500"
        disabled
        on:click={() => cancelPrint()}
      >
        <div class="self-center px-1"><Fa icon={faStop} /></div>
        <div class="flex-grow pr-1 text-end">Cancel</div>
      </button>
    </span>

    <span class="flex flex-col justify-center gap-1">
      <button class="flex rounded border-l-4 border-purple-400 bg-neutral-600 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => goto('/toolhead')}>
        <div class="self-center px-1"><Fa icon={faArrowsUpDownLeftRight} /></div>
        <div class="flex-grow pr-1 text-end">Toolhead</div>
      </button>

      <button class="flex rounded border-l-4 border-fuchsia-400 bg-neutral-600 px-1 py-2 text-white hover:bg-neutral-500" on:click={() => goto('/zoffset')}>
        <div class="self-center px-1"><Fa icon={faArrowsUpDown} /></div>
        <div class="flex-grow pr-1 text-end">Z-Offset</div>
      </button>
    </span>
    <span class="flex flex-col justify-center gap-1">
      <button
        class="flex rounded border-l-4 border-red-400 bg-neutral-600 px-1 py-2 text-white shadow hover:bg-neutral-500 disabled:text-neutral-500"
        disabled
        on:click={() => emergencyStop()}
      >
        <div class="self-center px-1"><Fa icon={faSkull} /></div>
        <div class="flex-grow pr-1 text-end">Kill</div>
      </button>
    </span>
  </div>
</div>
