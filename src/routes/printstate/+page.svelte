<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import type { IJsonRpcSuccessResponse } from '$lib/jsonrpc/types/IJsonRpcSuccessResponse';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import type { IFileMetadata } from '$lib/moonraker/types/IFileMetadata';
  import type { IThumbnail } from '$lib/moonraker/types/IThumbnail';

  let maxAcceleration = moonraker.toolhead.MaxAcceleration;
  let toolheadPosition = moonraker.toolhead.Position;
  let nozzleTemp = moonraker.extruder.Temperature;
  // let nozzleTemp = writable(240);
  let bedTemp = moonraker.heaterBed.Temperature;
  let nozzleTarget = moonraker.extruder.Target;
  // let nozzleTarget = writable(200);
  let pressureAdvance = moonraker.extruder.PressureAdvance;
  let bedTarget = moonraker.heaterBed.Target;
  let fanSpeed = moonraker.fan.Speed;
  let baby = moonraker.gcodeMove.HomeOrigin;
  let speed = moonraker.gcodeMove.Speed;
  let speedFactor = moonraker.gcodeMove.SpeedFactor;
  let extrudeFactor = moonraker.gcodeMove.ExtrudeFactor;

  let printStatsState = moonraker.printStats.State;
  // let printStatsState = writable<TPrintState>('printing');
  let printStatsMessage = moonraker.printStats.Message;
  let printStatsFilename = moonraker.printStats.Filename;
  // let printStatsFilename = writable('slejslkghdfkjghdkfjgjdfkjghdölfkgjdfölkgjdföljkbgxclvkbjdfkiolgjhsdlötkjzröstlkzhjfölgkhj');
  let printStatsPrintDuration = moonraker.printStats.PrintDuration;
  // let printStatsPrintDuration = writable(60);
  let filamentUsed = moonraker.printStats.FilamentUsed;
  let currentLayer = moonraker.printStats.Info.CurrentLayer;
  let totalLayer = moonraker.printStats.Info.TotalLayer;

  let progress = moonraker.displayStatus.Progress;
  // let progress = writable(0.25);
  let selectedFile = ''; // writable($printStatsFilename);

  let filamentTotal = 0;
  let remainingDuration = 0;
  let eta = new Date(Date.now()).toLocaleTimeString();
  let localTime: string = calcClock();

  function calcPrintRemainingTimes() {
    if ($progress <= 0.0) {
      //TODO from Metadata
      remainingDuration = 0;
      eta = '-';
    } else {
      remainingDuration = Math.round($printStatsPrintDuration / $progress - $printStatsPrintDuration);
      eta = new Date((Math.floor(Date.now() / 1000.0) + remainingDuration) * 1000).toLocaleTimeString();
    }
  }

  function calcClock(): string {
    return new Date(Date.now()).toLocaleTimeString();
  }

  setInterval(() => {
    calcPrintRemainingTimes();
    localTime = calcClock();
  }, 1000);

  $: {
    if ($printStatsFilename !== '') {
      selectedFile = $printStatsFilename;
    }
  }

  let selectedFileMeta: IFileMetadata;
  let selectedFileThumbnailPath: string;

  async function updateSelectedFileMeta(filename: string) {
    let resumeRequest = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: filename + '.gcode'
      }
    });
    let x = (await client.sendRequest(resumeRequest)) as IJsonRpcSuccessResponse;
    selectedFileMeta = x.result as IFileMetadata;
    let thumbnail: IThumbnail;
    // get thumbnail with largest width
    if (Array.isArray(selectedFileMeta.thumbnails) && selectedFileMeta.thumbnails.length > 0) {
      thumbnail = selectedFileMeta.thumbnails.sort((n1, n2) => n2.width - n1.width)[0];
      selectedFileThumbnailPath = import.meta.env.VITE_MOONRAKER_API + 'server/files/gcodes/' + thumbnail.relative_path;
    } else {
      selectedFileThumbnailPath = '';
    }

    remainingDuration = selectedFileMeta.estimated_time;
    filamentTotal = selectedFileMeta.filament_total;
  }

  $: {
    if (selectedFile != '') {
      updateSelectedFileMeta(selectedFile);
    }
  }

  async function startPrint() {
    let resumeRequest = new JsonRpcRequest({
      method: 'printer.print.start',
      params: {
        filename: selectedFile + '.gcode'
      }
    });
    await client.sendRequest(resumeRequest);
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

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex flex-grow flex-row justify-between">
    <span class="flex flex-grow flex-row justify-around">
      <div class="flex flex-col items-stretch justify-center gap-2">
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-600 px-2 py-2">
          <table class="table-auto text-sm text-neutral-50">
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="text-start">{$nozzleTemp.toFixed(1)} / {$nozzleTarget} °C</td>
            </tr>
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Bed</td>
              <td class="text-startt">{$bedTemp.toFixed(1)} / {$bedTarget} °C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Fan</td>
              <td class="text-start">{($fanSpeed * 100.0).toFixed(1)} %</td>
            </tr>
            {#if $printStatsState === 'printing'}
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="text-start">{($speed / 60.0).toFixed(0)} mm/s</td>
              </tr>
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Flow</td>
                <td class="text-start">21.0 mm³/s</td>
              </tr>
            {/if}
          </table>
        </div>
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-600 px-2 py-2">
          <table class="table-auto text-sm text-neutral-50">
            {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'error'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">X</td>
                <td class="text-start">{$toolheadPosition[0].toFixed(2)} mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Y</td>
                <td class="text-start">{$toolheadPosition[1].toFixed(2)} mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Z</td>
                <td class="text-start">{$toolheadPosition[2].toFixed(2)} mm</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Baby</td>
                <td class="text-start">{$baby.toFixed(3)} mm</td>
              </tr>
            {:else if $printStatsState === 'printing' || $printStatsState === 'paused'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="text-start">{($speedFactor * 100.0).toFixed(0)} %</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Extrude</td>
                <td class="text-start">{($extrudeFactor * 100.0).toFixed(0)} %</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Accel</td>
                <td class="text-start">{$maxAcceleration.toFixed(0)} mm/s²</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Pressure</td>
                <td class="text-start">{$pressureAdvance.toFixed(3)} x</td>
              </tr>
            {/if}
          </table>
        </div>
      </div>
      {#if selectedFile !== '' || $printStatsState !== 'standby'}
        <div class="flex flex-col items-stretch justify-center gap-2">
          <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
            {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'printing' || $printStatsState === 'paused' || $printStatsState === 'error'}
              <p class="overflow-clip w-36 text-center text-sm text-neutral-50">{selectedFile.substring(0, 20)}</p>
              <img src="{selectedFileThumbnailPath}" alt="{selectedFile}" loading="lazy" class="h-24 justify-center rounded-lg" />
            {/if}

            <table class="table-auto text-sm text-neutral-50">
              {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'error'}
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Remaining</td>
                  <td>{Math.round(remainingDuration / 60)} min</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td>{(filamentTotal / 1000.0).toFixed(1)} m</td>
                </tr>
              {:else if $printStatsState === 'printing' || $printStatsState === 'paused'}
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">ETA</td>
                  <td>{eta}</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Remains</td>
                  <td>{remainingDuration} s</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Progress</td>
                  <td>{($progress * 100.0).toFixed(1)} %</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Filament</td>
                  <td>{($filamentUsed / 1000.0).toFixed(1)} m</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Layer</td>
                  <td>{$currentLayer} / {$totalLayer}</td>
                </tr>
              {/if}
            </table>
          </div>
        </div>
      {/if}
    </span>
    <div class="flex flex-col items-end justify-center gap-3">
      {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete'}
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Load
        </button>
        {#if selectedFile !== ''}
          <button
            on:click="{startPrint}"
            class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
            Start
          </button>
        {/if}
      {:else if $printStatsState === 'printing'}
        <button
          on:click="{pausePrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Pause
        </button>
        <button
          on:click="{cancelPrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Cancel
        </button>
      {:else if $printStatsState === 'paused'}
        <button
          on:click="{resumePrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Continue
        </button>
        <button
          on:click="{cancelPrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
          Cancel
        </button>
      {/if}
    </div>
  </div>

  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
      State
    </button>
    {#if $printStatsState !== 'printing'}
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
    {#if $printStatsState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500">
        Prep
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{localTime}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-800 drop-shadow-md hover:bg-neutral-500"
      on:click="{emergencyStop}">
      Kill
    </button>
  </div>
</div>
