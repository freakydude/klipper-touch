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
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-500 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Load
        </button>
        {#if $printStatsFilename !== ''}
          <button
            on:click|preventDefault="{() => commands.startPrint($printStatsFilename)}"
            class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Start
          </button>
        {/if}
      {:else if $printStatsState === 'printing'}
        <button
          on:click|preventDefault="{() => commands.pausePrint()}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Pause
        </button>
        <button
          on:click|preventDefault="{() => (confirmCancelPrint = true)}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Cancel
        </button>
      {:else if $printStatsState === 'paused'}
        <button
          on:click|preventDefault="{() => commands.resumePrint()}"
          class="flex h-14 w-full items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Continue
        </button>
        <button
          on:click|preventDefault="{() => (confirmCancelPrint = true)}"
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
          on:click|preventDefault="{() => {
            commands.cancelPrint();
            confirmCancelPrint = false;
          }}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-red-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-neutral-500 disabled:opacity-50">
          Cancel
        </button>
        <button
          on:click|preventDefault="{() => (confirmCancelPrint = false)}"
          class="flex w-1/2 items-center justify-center rounded-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Abort
        </button>
      </span>
    </div>
  </div>
{/if}
