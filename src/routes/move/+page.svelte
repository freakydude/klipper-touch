<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  let printStatsState = moonraker.printStats.State;
  let motionReportLivePosition = moonraker.motionReport.LivePosition;
  let toolheadAxisMaximum = moonraker.toolhead.AxisMaximum;
  let toolheadAxisMinimum = moonraker.toolhead.AxisMinimum;
  let toolheadPosition = moonraker.toolhead.Position;
  let toolheadHomedAxes = moonraker.toolhead.HomedAxes;

  let localTime: string = calcClock();

  let stepsArr = [0.1, 1, 2, 5, 10, 20, 50, 100];
  let selectedStep = 6;
  let speedXY = 50;
  let speedZ = 20;

  let isHomedXY = false;
  let isHomedZ = false;

  $: {
    isHomedXY = $toolheadHomedAxes.includes('xy');
    isHomedZ = $toolheadHomedAxes.includes('z');
  }

  function calcClock(): string {
    return new Date(Date.now()).toLocaleTimeString('de', { timeStyle: 'short' });
  }

  setInterval(() => {
    localTime = calcClock();
  }, 1000);

  async function emergencyStop() {
    let emergencyStopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(emergencyStopRequest);
  }

  function getAbsolutePosition(axis: number, relativeSteps: number): number {
    let position = $toolheadPosition[axis] + relativeSteps;
    position = Math.max(position, $toolheadAxisMinimum[axis]);
    position = Math.min(position, $toolheadAxisMaximum[axis]);

    return position;
  }

  async function moveX(relativeSteps: number) {
    let targetZ = getAbsolutePosition(0, relativeSteps);
    console.log(targetZ);

    let moveDownRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G90\nG0 X' + targetZ + ' F' + speedXY * 60
      }
    });
    await client.sendRequest(moveDownRequest);
  }

  async function moveY(relativeSteps: number) {
    let targetY = getAbsolutePosition(1, relativeSteps);
    console.log(targetY);

    let moveDownRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G90\nG0 Y' + targetY + ' F' + speedXY * 60
      }
    });
    await client.sendRequest(moveDownRequest);
  }

  async function moveZ(relativeSteps: number) {
    let targetZ = getAbsolutePosition(2, relativeSteps);
    console.log(targetZ);

    let moveDownRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G90\nG0 Z' + targetZ + ' F' + speedZ * 60
      }
    });
    await client.sendRequest(moveDownRequest);
  }

  async function homeXY() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'G28 X Y'
      }
    });
    await client.sendRequest(homeRequest);
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

  async function disableSteppers() {
    let homeRequest = new JsonRpcRequest({
      method: 'printer.gcode.script',
      params: {
        script: 'M84'
      }
    });
    await client.sendRequest(homeRequest);
  }
</script>

<div class="page-dark flex-col items-stretch">
  <div class="flex w-full flex-row items-center justify-end gap-3 p-1">
    <p class="text-sm text-neutral-50">M117 Status Message</p>
  </div>
  <div class="flex flex-row">
    <div class="flex flex-grow justify-center">
      <div class="flex flex-col items-stretch justify-center gap-2 rounded-lg bg-neutral-600 px-2 py-2">
        <table class="w-80 self-center text-sm text-neutral-50">
          <tr class="border-b border-neutral-800">
            <td class="pr-2 text-end">Current</td>
            <td class="pr-1 text-start">X {$motionReportLivePosition[0].toFixed(2)} mm</td>
            <td class="pr-1 text-start">Y {$motionReportLivePosition[1].toFixed(2)} mm</td>
            <td class="pr-1 text-start">Z {$motionReportLivePosition[2].toFixed(2)} mm</td>
          </tr>
          <tr>
            <td class="pr-2 text-end">Target</td>
            <td class="pr-1 text-start">X {$toolheadPosition[0].toFixed(1)} mm</td>
            <td class="pr-1 text-start">Y {$toolheadPosition[1].toFixed(1)} mm</td>
            <td class="pr-1 text-start">Z {$toolheadPosition[2].toFixed(1)} mm</td>
          </tr>
        </table>

        <span class="flex flex-row items-center justify-center gap-3">
          <button
            disabled="{!isHomedZ}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
            on:click="{() => moveZ(-stepsArr[selectedStep])}">
            Down
          </button>
          <button
            disabled="{!isHomedXY}"
            on:click="{() => moveY(stepsArr[selectedStep])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
            Back
          </button>
          <button
            disabled="{!isHomedZ}"
            on:click="{() => moveZ(stepsArr[selectedStep])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
            Up
          </button>
        </span>

        <span class="flex flex-row items-center justify-center gap-3">
          <button
            disabled="{!isHomedXY}"
            on:click="{() => moveX(-stepsArr[selectedStep])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
            Left
          </button>

          <button
            disabled="{!isHomedXY}"
            on:click="{() => moveY(-stepsArr[selectedStep])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
            Front
          </button>
          <button
            disabled="{!isHomedXY}"
            on:click="{() => moveX(stepsArr[selectedStep])}"
            class="flex h-14 w-20 items-center justify-center rounded-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
            Right
          </button>
        </span>
      </div>
    </div>
    <span class="flex flex-col">
      <button
        on:click="{() => disableSteppers()}"
        class="flex h-10 w-20 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Off
      </button>
      <span class="flex flex-grow flex-col justify-end gap-3">
        <button
          on:click="{() => homeXY()}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg {isHomedXY
            ? 'bg-neutral-500'
            : 'bg-neutral-700'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          HomeXY
        </button>
        <button
          on:click="{() => homeZ()}"
          class="flex h-14 w-20 items-center justify-center rounded-l-lg {isHomedZ
            ? 'bg-neutral-500'
            : 'bg-neutral-700'}  px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
          HomeZ
        </button>
      </span>
    </span>
  </div>
  <span class="flex flex-grow flex-row items-center justify-center py-2">
    <div class="flex items-center gap-1 rounded-lg bg-neutral-700 pl-3">
      <p class="flex pr-1 text-neutral-50">Step</p>
      {#each stepsArr as number, i}
        <button
          class="hover:bg-neutral-000 flex w-12 items-center justify-center rounded-lg px-3 py-2 font-semibold text-neutral-50 drop-shadow-md disabled:opacity-50 {i ===
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
  <div class="flex flex-row gap-x-1 bg-neutral-700 px-1 pb-1">
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
      on:click="{async () => goto('/printstate')}">
      State
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-500 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Move
      </button>
    {/if}
    <button
      on:click="{async () => goto('/temperature')}"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
      Temp
    </button>
    <button
      on:click="{async () => goto('/babysteps')}"
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
      Baby
    </button>
    {#if $printStatsState !== 'printing'}
      <button
        disabled="{true}"
        class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50">
        Prep
      </button>
    {/if}
    <div class="flex flex-grow items-end justify-end">
      <p class="pb-1 pr-1 text-sm text-neutral-50">{localTime}</p>
    </div>
    <button
      class="flex w-16 items-center justify-center rounded-b-lg bg-neutral-600 px-3 py-2 font-semibold text-red-700 drop-shadow-md hover:bg-neutral-500 disabled:opacity-50"
      on:click="{emergencyStop}">
      Kill
    </button>
  </div>
</div>