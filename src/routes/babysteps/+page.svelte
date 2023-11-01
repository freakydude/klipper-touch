<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, clock, clockFormatter, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  let printStatsState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;
  let gcodeMoveHomingOrigin = moonraker.gcodeMove.HomeOrigin;

  let stepsArr = [0.005, 0.01, 0.02, 0.05, 0.1, 0.2, 0.5];
  let selectedStep = 3;

  let isHomedXY = false;
  let isHomedZ = false;

  $: {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  }

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }

  async function changeOffset(relativeSteps: number) {
    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z_ADJUST=' + relativeSteps + ' MOVE=1'
      }
    });
    await client.sendRequest(request);
  }

  async function resetOffset() {
    let request = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'SET_GCODE_OFFSET Z=0 MOVE=1'
      }
    });
    await client.sendRequest(request);
  }

  async function homeZ() {
    let homeRequest;
    if (!isHomedXY) {
      homeRequest = new JsonRpcRequest({
        method: 'printer.gcode.script',
        params: {
          script: 'G28'
        }
      });
    } else {
      homeRequest = new JsonRpcRequest({
        method: 'printer.gcode.script',
        params: {
          script: 'G28 Z'
        }
      });
    }
    await client.sendRequest(homeRequest);
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex w-full flex-row items-center justify-end gap-3 p-1">
    <p class="text-sm text-neutral-50">M117 Status Message</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-evenly">
      <div class="flex flex-col items-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
        <table class="self-stretch text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="px-2 text-end">Z</td>
            <td class=" text-start">{$motionReportLivePosition[2].toFixed(3)} mm</td>
          </tr>
          <tr>
            <td class="px-2 text-end">Z Offset</td>
            <td class=" text-start">{$gcodeMoveHomingOrigin[2].toFixed(3)} mm</td>
          </tr>
        </table>

        <button
          on:click="{() => changeOffset(stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
          Up
        </button>

        <button
          on:click="{() => changeOffset(-stepsArr[selectedStep])}"
          class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
          Down
        </button>
      </div>
    </div>
    <span class="flex flex-col">
      <span class="flex flex-grow flex-col justify-start gap-2">
        <button
          class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50"
          on:click="{() => {
            resetOffset();
          }}">
          Reset
        </button>
      </span>
      <span class="flex flex-grow flex-col justify-end gap-2">
        <button
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50"
          on:click="{() => {
            homeZ();
          }}">
          HomeZ
        </button>
        <button
          disabled="{true}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
          Save
        </button>
      </span>
    </span>
  </div>
  <span class="flex flex-grow flex-row items-center justify-center py-2">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50 {i ===
          selectedStep
            ? 'bg-neutral-500'
            : 'bg-neutral-600'} "
          on:click="{() => {
            selectedStep = i;
          }}">
          {number}
        </button>
      {/each}
    </div>
  </span>
  <!-- Nav -->
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50"
      on:click="{() => goto('/printstate')}">
      State
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        on:click="{() => goto('/move')}"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
        Move
      </button>
    {/if}
    <button
      on:click="{() => goto('/temperature')}"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
      Temp
    </button>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
      Baby
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        disabled="{true}"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500 disabled:opacity-50">
        Prep
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{clockFormatter.format($clock)}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md active:bg-blue-500 disabled:opacity-50"
      on:click="{emergencyStop}">
      Kill
    </button>
  </div>
</div>
