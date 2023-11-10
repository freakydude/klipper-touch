<script lang="ts">
  import { env } from '$env/dynamic/public';
  import { client, commands, moonraker, values } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import type { IFileMetadata } from '$lib/moonraker/types/IFileMetadata';
  import type { IThumbnail } from '$lib/moonraker/types/IThumbnail';

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

  let printStatsState = moonraker.printStats.State;
  let printStatsFilename = moonraker.printStats.Filename;
  let printStatsPrintDuration = moonraker.printStats.PrintDuration;
  let filamentUsed = moonraker.printStats.FilamentUsed;
  let currentLayer = moonraker.printStats.Info.CurrentLayer;
  let totalLayer = moonraker.printStats.Info.TotalLayer;
  let displayStatusMessage = moonraker.displayStatus.Message;

  let clockFormatter = values.clockFormatter;
  let clock = values.clock;

  let progress = moonraker.displayStatus.Progress;
  let selectedFile = '';
  let selectedFileThumbnailPath = '';
  let remainingDuration = 0;
  let filamentTotal = 0;
  let estimatedTime = 0;
  let layerHeight = 0;
  let objectHeight = 0;
  let eta = '';

  let statusLine: string;

  $: {
    statusLine = $displayStatusMessage + (selectedFile !== '' ? ' - ' + selectedFile : '');
  }

  $: updatePropertiesOnPrintingFile($printStatsFilename);

  function updatePropertiesOnPrintingFile(name: string) {
    if (name !== '') {
      selectedFile = name;

      let meta = getFileMetadata(selectedFile + '.gcode');
      meta.then(async (m) => {
        if (m !== null) {
          estimatedTime = m.estimated_time;
          filamentTotal = m.filament_total;
          layerHeight = m.layer_height;
          objectHeight = m.object_height;
          selectedFileThumbnailPath = await getSelectedFileThumbnailPath(m.thumbnails);
        }
      });
    }
  }

  $: updateEta($progress);

  function updateEta(progress: number) {
    console.log('progress', progress, 'duration', $printStatsPrintDuration);
    if (progress > 0 && $printStatsPrintDuration > 60) {
      // wait 60sec before update eta dynamic
      remainingDuration = Math.floor($printStatsPrintDuration / progress - $printStatsPrintDuration);
      eta = clockFormatter.format(new Date((Math.floor(Date.now() / 1000.0) + remainingDuration) * 1000));
    } else {
      remainingDuration = estimatedTime;
      eta = clockFormatter.format(new Date((Math.floor(Date.now() / 1000.0) + remainingDuration) * 1000));
    }
    console.log('eta', eta);
  }

  async function getSelectedFileThumbnailPath(thumbnails: IThumbnail[]): Promise<string> {
    let path = '';
    if (Array.isArray(thumbnails) && thumbnails.length > 0) {
      let thumbnail = thumbnails.sort((n1, n2) => n2.width - n1.width)[0];
      path = (env.PUBLIC_KT_MOONRAKER_API === undefined ? 'http://127.0.0.1' : env.PUBLIC_KT_MOONRAKER_API) + '/server/files/gcodes/' + thumbnail.relative_path;
    } else {
      path = '';
    }
    return path;
  }

  async function getFileMetadata(filename: string): Promise<IFileMetadata | null> {
    let requestMetadata = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: filename
      }
    });
    let response = await client.sendRequest(requestMetadata);
    let metadata: IFileMetadata | null = null;

    if (response.error === undefined) {
      metadata = response.result as IFileMetadata;
    } else {
      return Promise.reject(response.error);
    }

    return metadata;
  }
</script>

<div class="page-dark flex-col items-stretch">
  <!-- Status line -->
  <div class="flex w-full flex-row items-center justify-center gap-3 p-1">
    <p class="text-sm text-neutral-50">{statusLine}</p>
  </div>
  <div class="flex flex-grow flex-row justify-between">
    <span class="flex flex-grow flex-row justify-around p-1">
      <div class="flex flex-col justify-around gap-2">
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-700 p-2">
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
                <td class="text-start">{$liveVelocity.toFixed(0)} mm/s</td>
                <!-- {$requestedSpeed.toFixed(0)} -->
              </tr>
              <tr class="border-t border-neutral-800">
                <td class="pr-2 text-end">Flow</td>
                <td class="text-start">{(Math.pow(1.75 / 2, 2) * Math.PI * $liveExtruderVelocity).toFixed(1)} mm³/s</td>
              </tr>
            {/if}
          </table>
        </div>
        <div class="flex flex-col items-stretch rounded-lg bg-neutral-700 p-2">
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
                <td class="text-start">{$baby[2].toFixed(3)} mm</td>
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
          <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-700 pb-2">
            {#if !selectedFileThumbnailPath}
              <p
                class="flex w-32 items-center justify-center rounded-lg border-2 border-neutral-700 bg-neutral-800 p-3 text-center text-xl font-extrabold text-neutral-400">
                No Preview
              </p>
            {:else if $printStatsState === 'standby' || $printStatsState === 'cancelled' || $printStatsState === 'complete' || $printStatsState === 'printing' || $printStatsState === 'paused' || $printStatsState === 'error'}
              <img
                class="flex h-32 justify-center rounded-lg border-2 border-neutral-700"
                loading="lazy"
                src="{selectedFileThumbnailPath}"
                alt="{selectedFile}" />
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
                  <td class="pr-2 text-end">Layer</td>
                  <td>{$currentLayer} / {$totalLayer}</td>
                </tr>
                <!-- <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Remains</td>
                  <td>{remainingDuration} s</td>
                </tr> -->
                <tr class="border-b border-neutral-800">
                  <td class="pr-2 text-end">Progress</td>
                  <td>{($progress * 100.0).toFixed(1)} %</td>
                </tr>
                <tr>
                  <td class="pr-2 text-end">Filament</td>
                  <td>{($filamentUsed / 1000.0).toFixed(1)} m</td>
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
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-500 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Load
        </button>
        {#if selectedFile !== ''}
          <button
            on:click|preventDefault="{() => commands.startPrint(selectedFile + '.gcode')}"
            class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
            Start
          </button>
        {/if}
      {:else if $printStatsState === 'printing'}
        <button
          on:click|preventDefault="{commands.pausePrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Pause
        </button>
        <button
          on:click|preventDefault="{commands.cancelPrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Cancel
        </button>
      {:else if $printStatsState === 'paused'}
        <button
          on:click|preventDefault="{commands.resumePrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Continue
        </button>
        <button
          on:click|preventDefault="{commands.cancelPrint}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
          Cancel
        </button>
      {/if}
    </div>
  </div>

  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      State
    </button>
    {#if $printStatsState !== 'printing'}
      <a
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50"
        href="/move">
        Move
      </a>
    {/if}
    <a
      href="/temperature"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Temp
    </a>
    <a
      href="/babysteps"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      Baby
    </a>
    {#if $printStatsState !== 'printing'}
      <a
        href="/extrusion"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Extr
      </a>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      on:click|preventDefault="{commands.emergencyStop}">
      Kill
    </button>
  </div>
</div>
