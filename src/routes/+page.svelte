<script lang="ts">
import { goto } from '$app/navigation';
import { client, moonraker } from '$lib/base.svelte';
import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
import { onMount } from 'svelte';

let isFullscreen = false;
let outerElement: Element;

let isConnected = client.isConnected;
let klippyState = moonraker.klippyState.state;
let klippyStateMessage = moonraker.klippyState.message;

async function reconnectToMoonraker() {
  await moonraker.disconnect();
  await moonraker.connect();
}

async function printerRestart() {
  let stopRequest = new JsonRpcRequest({
    method: 'printer.restart',
    params: {}
  });
  await client.sendRequest(stopRequest);
}

async function firmwareRestart() {
  let stopRequest = new JsonRpcRequest({
    method: 'printer.firmware_restart',
    params: {}
  });
  await client.sendRequest(stopRequest);
}

onMount(async () => {
  // await reconnectToMoonraker();
});
</script>

<div class="page-dark flex-col flex-wrap" bind:this="{outerElement}">
  {#if $isConnected}
    <p class="label1">Moonraker: connected</p>

    {#if $klippyState === 'disconnected'}
      <p class="label1">Firmware: disconnected</p>
      <p class="label1">{$klippyStateMessage}</p>
      <button class="btn-touch1" on:click="{() => reconnectToMoonraker()}">Reconnect</button>
    {:else if $klippyState === 'startup'}
      <p class="label1">Firmware: startup</p>
      <p class="label1">{$klippyStateMessage}</p>
      <button class="btn-touch1" on:click="{() => printerRestart()}">Restart</button>
      <button class="btn-touch1" on:click="{() => firmwareRestart()}">Firmware Restart</button>
    {:else if $klippyState === 'ready'}
      <p class="label1">Firmware: ready</p>
      <!-- <p class="label1">{$klippyStateMessage}</p> -->
      <!-- <button class="btn-touch1" on:click="{() => printerRestart()}">Restart</button> -->
      <button class="btn-touch1" on:click="{() => goto('/printstate')}">Printer state</button>
    {:else if $klippyState === 'shutdown'}
      <p class="label1">Firmware: shutdown</p>
      <p class="label1">{$klippyStateMessage}</p>
      <button class="btn-touch1" on:click="{() => printerRestart()}">Restart</button>
      <button class="btn-touch1" on:click="{() => firmwareRestart()}">Firmware Restart</button>
    {:else if $klippyState === 'error'}
      <p class="label1">Firmware: error</p>
      <p class="label1">{$klippyStateMessage}</p>
      <button class="btn-touch1" on:click="{() => printerRestart()}">Restart</button>
      <button class="btn-touch1" on:click="{() => firmwareRestart()}">Firmware Restart</button>
    {/if}
  {:else}
    <p class="label1">Moonraker: disconnected</p>
    <button class="btn-touch1" on:click="{() => reconnectToMoonraker()}">Reconnect</button>
    <!-- <button class="btn-touch1" on:click="{() => switchFullscreen()}">Fullscreen</button> -->
  {/if}
</div>
