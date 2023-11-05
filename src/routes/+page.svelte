<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';
  import { onMount } from 'svelte';

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
</script>

<div class="page-dark flex-row content-between items-stretch gap-3">
  <p class="fixed bottom-1 right-1 text-sm text-neutral-100">v0.1.0 experimental</p>

  <div class="flex flex-grow flex-col justify-around p-2">
    <span class="flex flex-col items-center">
      <h1 class="text-center text-4xl font-bold text-neutral-100">Klipper-Touch</h1>
      <h1 class="text-center text-xl text-red-600">by freakyDude</h1>
    </span>
    <span class="flex flex-col items-center p-2">
      <table class="table-auto">
        {#if $isConnected}
          <tr>
            <td class="pr-2 text-end text-neutral-100">Moonraker</td>
            <td class="text-green-600">Connected</td>
          </tr>
          {#if $klippyState === 'disconnected'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klipper</td>
              <td class="text-purple-600">Disconnected</td>
            </tr>
          {:else if $klippyState === 'startup'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klipper</td>
              <td class="text-yellow-600">Startup</td>
            </tr>
          {:else if $klippyState === 'ready'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klipper</td>
              <td class="text-green-600">Ready</td>
            </tr>
          {:else if $klippyState === 'shutdown'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klipper</td>
              <td class="text-orange-600">Shutdown</td>
            </tr>
          {:else if $klippyState === 'error'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klipper</td>
              <td class="text-red-600">Error</td>
            </tr>
          {/if}
        {:else}
          <tr>
            <td class="pr-2 text-end text-neutral-100">Moonraker</td>
            <td class="text-purple-600">Disconnected </td>
          </tr>
        {/if}
      </table>
    </span>
    <span class="flex flex-col p-2 text-center">
      {#if $isConnected}
        {#if $klippyState === 'disconnected'}
          <p class="flex text-neutral-100">{$klippyStateMessage}</p>
        {:else if $klippyState === 'startup'}
          <p class="flex text-neutral-100">{$klippyStateMessage}</p>
        {:else if $klippyState === 'shutdown'}
          <p class="flex text-neutral-100">{$klippyStateMessage}</p>
        {:else if $klippyState === 'error'}
          <p class="flex text-neutral-100">{$klippyStateMessage}</p>
        {/if}
      {/if}
    </span>
  </div>
  <div class="flex flex-col justify-around">
    {#if $isConnected}
      {#if $klippyState !== 'ready'}
        <button
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500"
          on:pointerdown|preventDefault="{() => printerRestart()}">Restart</button>
        <button
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500"
          on:pointerdown|preventDefault="{() => firmwareRestart()}">Firmware Restart</button>
      {:else}
        <button
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500"
          on:pointerdown|preventDefault="{() => goto('/printstate')}">Printer State</button>
      {/if}
    {:else}
      <button
        class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-blue-500"
        on:pointerdown|preventDefault="{() => reconnectToMoonraker()}">Reconnect</button>
    {/if}
  </div>
</div>
