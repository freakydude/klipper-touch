<script lang="ts">
  import { bootParams, commands, moonraker, values } from '$lib/base.svelte';
  import BottomNavigation from '$lib/BottomNavigation.svelte';
  import StatusLine from '$lib/StatusLine.svelte';

  let maxAcceleration = moonraker.toolhead.MaxAcceleration;
  let toolheadPosition = moonraker.motionReport.LivePosition;
  let nozzleTemp = moonraker.extruder.Temperature;
  let bedTemp = moonraker.heaterBed.Temperature;
  let nozzleTarget = moonraker.extruder.Target;
  let pressureAdvance = moonraker.extruder.PressureAdvance;
  let bedTarget = moonraker.heaterBed.Target;
  let fanSpeed = moonraker.fan.Speed;
  let baby = moonraker.gcodeMove.HomeOrigin;
  let liveVelocity = moonraker.motionReport.LiveVelocity;
  let liveExtruderVelocity = moonraker.motionReport.LiveExtruderVelocity;
  let speedFactor = moonraker.gcodeMove.SpeedFactor;
  let extrudeFactor = moonraker.gcodeMove.ExtrudeFactor;

  // let printStatsState = writable<TPrintState>('standby');
  let printStatsState = moonraker.printStats.State;
  let printStatsFilename = moonraker.printStats.Filename;
  let printStatsPrintDuration = moonraker.printStats.PrintDuration;
  let filamentUsed = moonraker.printStats.FilamentUsed;
  let currentLayer = moonraker.printStats.Info.CurrentLayer;
  let totalLayer = moonraker.printStats.Info.TotalLayer;
  let fileMeta = values.fileMetadata;
  let clockFormatter = values.clockFormatter;
  let progress = moonraker.displayStatus.Progress;
  let selectedFileThumbnailPath = values.largestAbsoluteThumbnailPath;
  let dynamicRemainingTime = 0;
  let dynamicEta = '';
  let filamentTotal = 0;
  let confirmCancelPrint = false;
  let printStartTime = 0;
  let estimatedTime = 0;
  let estimatedETA = ' ';
  let apiUrl = bootParams.moonrakerApi;

  let loadDialog = false;
  let sortedFileNames = ['File 1', 'File 2', 'File 3', 'File 4', 'File 5', 'File 6'];
  let thumbnails: string[] = [];
  let selectedFilename = 1;

  async function clickLoad() {
    loadDialog = true;
    let gcodeFiles = await commands.listFiles();

    sortedFileNames = gcodeFiles.sort((n1, n2) => n2.modified - n1.modified).map((file) => file.path);

    let tempThumbnails: string[] = new Array<string>(sortedFileNames.length);

    await Promise.all(
      sortedFileNames.map(async (gcodeFile) => {
        let x = await values.getThumbnails(gcodeFile);
        if (x !== null) {
          let thumbFile = await values.getLargestAbsoluteThumbnailPath($apiUrl, x);
          tempThumbnails[sortedFileNames.indexOf(gcodeFile)] = thumbFile; // TODO Dict instead of array
        }
      })
    );

    thumbnails = tempThumbnails;
  }

  $: {
    if ($fileMeta !== null) {
      filamentTotal = $fileMeta.filament_total;
      printStartTime = $fileMeta.print_start_time;
      estimatedTime = $fileMeta.estimated_time;
    }
  }

  $: {
    estimatedETA = clockFormatter.format(new Date((Math.floor(Date.now() / 1000.0) + estimatedTime) * 1000));
    console.log(estimatedETA);
    console.log(estimatedTime);
  }

  $: {
    updateEta($progress); // TODO update on pause
  }

  function updateEta(progress: number) {
    if (progress > 0 && $printStatsPrintDuration > 60) {
      // wait 60sec before update eta dynamic
      dynamicRemainingTime = Math.floor($printStatsPrintDuration / progress - $printStatsPrintDuration);
      dynamicEta = clockFormatter.format(new Date((Math.floor(Date.now() / 1000.0) + dynamicRemainingTime) * 1000));
    } else {
      dynamicEta = estimatedETA;
    }
  }
</script>

<div class="page-dark flex-col items-stretch justify-between gap-1">
  <StatusLine />
  <div class="flex h-full w-full flex-row justify-between gap-1">
    <span class="flex w-5/6 flex-row justify-around gap-1">
      <div class="flex flex-col items-stretch justify-around gap-2">
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-700 p-2">
          <table class="table-auto text-sm text-neutral-50">
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="w-28 text-start">{$nozzleTemp.toFixed(1)} / {$nozzleTarget} °C</td>
            </tr>
            <tr class="border-b border-neutral-800">
              <td class="pr-2 text-end">Bed</td>
              <td class="w-28 text-start">{$bedTemp.toFixed(1)} / {$bedTarget} °C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Fan</td>
              <td class="w-24 text-start">{($fanSpeed * 100.0).toFixed(1)} %</td>
            </tr>
            {#if $printStatsState === 'printing'}
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="w-24 text-start">{$liveVelocity.toFixed(0)} mm/s</td>
                <!-- {$requestedSpeed.toFixed(0)} -->
              </tr>
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Flow</td>
                <td class="w-24 text-start">{(Math.pow(1.75 / 2, 2) * Math.PI * $liveExtruderVelocity).toFixed(1)} mm³/s</td>
              </tr>
            {/if}
          </table>
        </div>
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-700 p-2">
          <table class="table-auto text-sm text-neutral-50">
            {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'error'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">X</td>
                <td class="w-24 text-start">{$toolheadPosition[0].toFixed(2)} mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Y</td>
                <td class="w-24 text-start">{$toolheadPosition[1].toFixed(2)} mm</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Z</td>
                <td class="w-24 text-start">{$toolheadPosition[2].toFixed(2)} mm</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">Baby</td>
                <td class="text-start">{$baby[2].toFixed(3)} mm</td>
              </tr>
            {:else if $printStatsState === 'printing' || $printStatsState === 'paused'}
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Speed</td>
                <td class="w-24 text-start">{($speedFactor * 100.0).toFixed(0)} %</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Extrude</td>
                <td class="w-24 text-start">{($extrudeFactor * 100.0).toFixed(0)} %</td>
              </tr>
              <tr class="border-b border-neutral-800">
                <td class="pr-2 text-end">Accel</td>
                <td class="w-24 text-start">{$maxAcceleration.toFixed(0)} mm/s²</td>
              </tr>
              <tr>
                <td class="pr-2 text-end">PA</td>
                <td class="w-24 text-start">{$pressureAdvance.toFixed(3)}</td>
              </tr>
            {/if}
          </table>
        </div>
      </div>
      {#if $printStatsFilename !== '' || $printStatsState !== 'standby'}
        <div class="flex flex-col items-center justify-center gap-2">
          <div class="flex flex-col items-center rounded-lg bg-neutral-700 pb-2">
            {#if $selectedFileThumbnailPath === ''}
              <p
                class="flex w-32 items-center justify-center rounded-lg border-2 border-neutral-700 bg-neutral-800 p-3 text-center text-xl font-extrabold text-neutral-400">
                No Preview
              </p>
            {:else if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'printing' || $printStatsState === 'paused' || $printStatsState === 'error'}
              <img
                class="flex h-32 justify-center rounded-lg border-2 border-neutral-700"
                loading="lazy"
                src="{$selectedFileThumbnailPath}"
                alt="{$printStatsFilename}" />
            {/if}
            <table class="table-auto text-sm text-neutral-50">
              {#if $printStatsState === 'standby'}
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">ETA</td>
                  <td class="text-start">{estimatedETA}</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td class="text-start">{(filamentTotal / 1000.0).toFixed(1)} m</td>
                </tr>
              {:else if $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'error'}
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Duration</td>
                  <td class="text-start">{Math.round($printStatsPrintDuration / 60)} min</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Layer</td>
                  <td class="text-start">{$currentLayer}</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Progress</td>
                  <td class="text-start">{($progress * 100.0).toFixed(1)} %</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td class="text-start">{($filamentUsed / 1000.0).toFixed(1)} m</td>
                </tr>
              {:else if $printStatsState === 'printing' || $printStatsState === 'paused'}
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">ETA</td>
                  <td class="text-start">{dynamicEta}</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Layer</td>
                  <td class="text-start">{$currentLayer} / {$totalLayer}</td>
                </tr>
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Progress</td>
                  <td class="text-start">{($progress * 100.0).toFixed(1)} %</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td class="text-start">{($filamentUsed / 1000.0).toFixed(1)} m</td>
                </tr>
              {/if}
            </table>
          </div>
        </div>
      {/if}
    </span>
    <div class="flex h-full w-1/6 flex-col items-end justify-around gap-3">
      {#if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete'}
        <button
          on:click="{() => clickLoad()}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Load
        </button>
        {#if $printStatsFilename !== ''}
          <button
            on:click="{() => commands.startPrint($printStatsFilename)}"
            class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Start
          </button>
        {/if}
      {:else if $printStatsState === 'printing'}
        <button
          on:click="{() => commands.pausePrint()}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Pause
        </button>
        <button
          on:click="{() => (confirmCancelPrint = true)}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Cancel
        </button>
      {:else if $printStatsState === 'paused'}
        <button
          on:click="{() => commands.resumePrint()}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Continue
        </button>
        <button
          on:click="{() => (confirmCancelPrint = true)}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Cancel
        </button>
      {/if}
    </div>
  </div>

  <BottomNavigation />
</div>
{#if confirmCancelPrint}
  <div class="absolute flex h-full w-full items-center justify-center bg-black bg-opacity-50">
    <div class="flex flex-col items-center justify-center gap-4 rounded-lg border-neutral-600 bg-neutral-700 bg-opacity-50 p-4 drop-shadow-md backdrop-blur">
      <p class=" text-center text-neutral-100">Are you sure you want to cancel the print?</p>
      <span class="flex w-1/2 gap-3">
        <button
          on:click="{() => {
            commands.cancelPrint();
            confirmCancelPrint = false;
          }}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-red-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-neutral-500 disabled:opacity-50">
          Cancel
        </button>
        <button
          on:click="{() => (confirmCancelPrint = false)}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Abort
        </button>
      </span>
    </div>
  </div>
{/if}

{#if loadDialog}
  <div class="absolute flex h-full w-full items-center justify-center bg-black bg-opacity-50 p-2">
    <div
      class="flex h-full w-full flex-row items-center justify-center gap-2 rounded-lg border-neutral-600 bg-neutral-700 bg-opacity-50 p-2 drop-shadow-md backdrop-blur">
      <div class="flex max-h-full w-5/6 flex-col gap-1 overflow-y-auto overflow-x-hidden pr-4">
        <table class="w-full table-auto gap-1 rounded-lg drop-shadow-md">
          {#each sortedFileNames as filename, i}
            <tr
              class="border-b border-neutral-800 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i === selectedFilename
                ? 'bg-neutral-500'
                : 'bg-neutral-600'}"
              on:click="{() => {
                selectedFilename = i;
              }}">
              <td class="">
                {#if thumbnails[i] === ''}
                  <div
                    class="w-16 items-stretch justify-center rounded-lg border-2 border-neutral-700 bg-neutral-800 p-3 text-center text-xl font-extrabold text-neutral-400">
                    ?
                  </div>
                {:else}
                  <img class="flex w-16 justify-center rounded-lg" loading="lazy" src="{thumbnails[i]}" alt="{$printStatsFilename}" />
                {/if}
              </td>
              <td class="px-1 py-1 text-sm text-neutral-50">
                {filename}
              </td>
            </tr>
          {/each}
        </table>
      </div>
      <div class="flex max-h-full w-1/5 flex-col items-stretch gap-2">
        <button
          on:click="{() => (loadDialog = false)}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Load
        </button>

        <button
          on:click="{() => (loadDialog = false)}"
          class="flex items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
