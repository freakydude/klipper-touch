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
  let metaFilamentTotal = 0;
  let confirmCancelPrint = false;
  let metaPrintStartTime = 0;
  let metaEstimatedTime = 0;
  let estimatedDate = Date.now();
  let apiUrl = bootParams.moonrakerApi;

  let loadDialog = false;
  let selectedFileEntry = 0;

  let divElement: HTMLDivElement;

  class FileListEntry {
    name: string = '';
    thumbnailUrl: string = '';
    weight: number = 0;
    duration: number = 0;
    modified: number = 0;
  }

  enum sortVariant {
    NameAsc,
    NameDesc,
    ModifiedAsc,
    ModifiedDesc
  }

  let currentSort: sortVariant = sortVariant.NameDesc;

  function cycleSortVariant() {
    if (currentSort === sortVariant.ModifiedDesc) {
      currentSort = sortVariant.NameAsc;
    } else {
      currentSort++;
    }
  }

  type sortFunc = (n1: FileListEntry, n2: FileListEntry) => number;

  let sortDic = new Map<sortVariant, sortFunc>([
    [sortVariant.NameAsc, (n1: FileListEntry, n2: FileListEntry) => (n1.name < n2.name ? -1 : 1)],
    [sortVariant.NameDesc, (n1: FileListEntry, n2: FileListEntry) => (n1.name > n2.name ? -1 : 1)],
    [sortVariant.ModifiedAsc, (n1: FileListEntry, n2: FileListEntry) => (n1.modified < n2.modified ? -1 : 1)],
    [sortVariant.ModifiedDesc, (n1: FileListEntry, n2: FileListEntry) => (n1.modified > n2.modified ? -1 : 1)]
  ]);

  let fileEntries: FileListEntry[] = [];

  async function clickLoad() {
    loadDialog = true;

    let gcodeFiles = await commands.listFiles();

    await Promise.all(
      gcodeFiles.map(async (file) => {
        let meta = await values.getFileMetadata(file.path);
        let index = gcodeFiles.indexOf(file);
        let entry: FileListEntry = new FileListEntry();
        if (meta !== null) {
          entry.duration = meta.estimated_time;
          entry.name = file.path;
          entry.weight = meta.filament_weight_total;
          entry.modified = meta.modified;
          entry.thumbnailUrl = await values.getLargestAbsoluteThumbnailPath($apiUrl, meta.thumbnails);
        }

        fileEntries[index] = entry;
      })
    );
  }

  let dateFormat = new Intl.DateTimeFormat('de', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    month: 'numeric',
    day: 'numeric',
    year: 'numeric'
  });

  $: {
    if ($fileMeta !== null) {
      metaFilamentTotal = $fileMeta.filament_total;
      metaPrintStartTime = $fileMeta.print_start_time;
      metaEstimatedTime = $fileMeta.estimated_time;
    }
  }

  $: {
    if ($fileMeta !== null) {
      let progressTime = $progress * metaEstimatedTime;
      estimatedDate = Date.now() + (metaEstimatedTime - progressTime) * 1000;
    } else {
      let totalTime = $printStatsPrintDuration / $progress;
      estimatedDate = Date.now() + (totalTime - $printStatsPrintDuration) * 1000;
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
                  <td class="text-start">{clockFormatter.format(estimatedDate)}</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td class="text-start">{(metaFilamentTotal / 1000.0).toFixed(1)} m</td>
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
                  <td class="text-start">{clockFormatter.format(estimatedDate)}</td>
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
  <div class="absolute flex h-full w-full items-center justify-center bg-black bg-opacity-50 pb-2 pl-2 pt-2">
    <div
      class="flex h-full w-full flex-row items-stretch justify-center gap-2 rounded-l-lg border-neutral-600 bg-neutral-700 bg-opacity-50 pb-1 pl-1 pt-1 drop-shadow-md backdrop-blur">
      <div class="flex max-h-full w-5/6 flex-col gap-1 overflow-y-auto overflow-x-hidden rounded-lg" bind:this="{divElement}">
        <table class="w-full table-auto gap-1 rounded-lg drop-shadow-md">
          {#each fileEntries.sort(sortDic.get(currentSort)) as entry, i}
            <tr
              class="border-b border-neutral-800 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i === selectedFileEntry
                ? 'bg-neutral-500'
                : 'bg-neutral-600'}"
              on:click="{() => {
                selectedFileEntry = i;
              }}">
              <td class="p-0.5">
                {#if entry.thumbnailUrl === ''}
                  <div class="w-16 content-center items-center justify-center rounded-lg bg-neutral-800 p-2 text-center text-xl font-extrabold text-red-500">
                    ?
                  </div>
                {:else}
                  <img class=" flex w-16 justify-center rounded-lg" loading="lazy" src="{entry.thumbnailUrl}" alt="{entry.name}" />
                {/if}
              </td>

              <td class="whitespace-nowrap">
                <div class="flex flex-col px-1 py-1">
                  <div class="pb-1 text-sm text-neutral-50">{entry.name.slice(0, entry.name.length - 6)}</div>
                  {#if entry.modified}
                    <div class="text-xs text-neutral-300">Modified: {dateFormat.format(new Date(Date.now() + entry.modified))}</div>
                  {/if}
                  <div class="flex flex-row">
                    {#if entry.duration}
                      <div class="pr-2 text-xs text-neutral-300">Duration: {(entry.duration / 60.0).toFixed(0)} min</div>
                    {/if}
                    {#if entry.weight}
                      <div class="text-xs text-neutral-300">Weight: {entry.weight.toFixed(0)} g</div>
                    {/if}
                  </div>
                </div>
              </td>
            </tr>
          {/each}
        </table>
      </div>
      <div class="flex max-h-full w-1/5 flex-col items-stretch justify-center gap-3">
        <button
          on:click="{() => {
            divElement.scrollBy({ top: (-divElement.clientHeight * 2) / 3.0, behavior: 'smooth' });
          }}"
          class="flex items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Up
        </button>
        <button
          on:click="{() => {
            divElement.scrollBy({ top: (+divElement.clientHeight * 2) / 3.0, behavior: 'smooth' });
          }}"
          class="flex items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Down
        </button>
        <button
          on:click="{() => {
            cycleSortVariant();
          }}"
          class="flex items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Sort
        </button>
        <button
          on:click="{() => {
            loadDialog = false;
          }}"
          class="flex items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Load
        </button>
        <button
          on:click="{() => {
            loadDialog = false;
          }}"
          class="flex items-center justify-center rounded-l-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
