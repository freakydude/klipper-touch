<script lang="ts">
  import { client, clock, clockFormatter, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  let printState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;
  let gcodeMoveHomingOrigin = moonraker.gcodeMove.HomeOrigin;
  let saveConfigPending = moonraker.configFile.SaveConfigPending;
  let liveExtruderVelocity = moonraker.motionReport.LiveExtruderVelocity;
  let nozzleTarget = moonraker.extruder.Target;
  let nozzleTemp = moonraker.extruder.Temperature;
  let canExtrude = moonraker.extruder.CanExtrude;

  let stepsArr = [1, 2, 5, 10, 20, 50, 100];
  let selectedStep = 3;
  let speedArr = [1, 2, 3, 5, 10];
  let selectedSpeed = 2;

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }

  async function extrude(relativeSteps: number, speed: number) {
    // let request = new JsonRpcRequest({
    //   method: 'printer.gcode.script',
    //   params: {
    //     script: 'SET_GCODE_OFFSET Z_ADJUST=' + relativeSteps + ' MOVE=1'
    //   }
    // });
    // await client.sendRequest(request);
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex w-full flex-row items-center justify-end gap-3 p-1">
    <p class="text-sm text-neutral-50">M117 Status Message</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-evenly">
      <div class="flex flex-col items-center gap-3">
        <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
          <table class="self-stretch text-sm text-neutral-50">
            <tr class="border-b border-neutral-800 {$canExtrude === false ? 'text-red-500' : ''}">
              <td class="pr-2 text-end">Nozzle</td>
              <td class="text-start">{$nozzleTemp.toFixed(1)} / {$nozzleTarget} °C</td>
            </tr>
            <tr>
              <td class="pr-2 text-end">Flow</td>
              <td class="text-start">{(Math.pow(1.75 / 2, 2) * Math.PI * $liveExtruderVelocity).toFixed(1)} mm³/s</td>
            </tr>
          </table>
          <span class="flex gap-3">
            <button
              disabled="{$printState === 'printing' || $canExtrude === false}"
              on:click|preventDefault="{() => extrude(stepsArr[selectedStep], speedArr[selectedSpeed])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Retract
            </button>

            <button
              disabled="{$printState === 'printing' || $canExtrude === false}"
              on:click|preventDefault="{() => extrude(-stepsArr[selectedStep], speedArr[selectedSpeed])}"
              class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
              Extrude
            </button></span>
        </div>

        <span class="flex flex-row items-center justify-center">
          <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
            <p class="flex pr-1 text-neutral-50">Speed</p>
            {#each speedArr as number, i}
              <button
                class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
                selectedSpeed
                  ? 'bg-neutral-500'
                  : 'bg-neutral-600'} "
                on:click|preventDefault="{() => {
                  selectedSpeed = i;
                }}">
                {number}
              </button>
            {/each}
          </div>
        </span>
      </div>
    </div>
    <span class="flex flex-col">
      <span class="flex flex-grow flex-col justify-start gap-2">
        <!-- Right side buttons here -->
      </span>
    </span>
  </div>
  <span class="flex flex-grow flex-row items-center justify-center py-2">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50 {i ===
          selectedStep
            ? 'bg-neutral-500'
            : 'bg-neutral-600'} "
          on:click|preventDefault="{() => {
            selectedStep = i;
          }}">
          {number}
        </button>
      {/each}
    </div>
  </span>
  <!-- Nav -->
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <a
      href="/printstate"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
      State
    </a>
    {#if $printState !== 'printing'}
      <a
        href="/move"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
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
    {#if $printState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500 disabled:opacity-50">
        Extr
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-red-500 disabled:opacity-50"
      on:click|preventDefault="{emergencyStop}">
      Kill
    </button>
  </div>
</div>
