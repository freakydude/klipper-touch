<script lang="ts">
  import { bootParams, client, commands, moonraker } from '$lib/base.svelte';

  let wsUrl = bootParams.moonrakerWs;

  let isConnected = client.isConnected;
  let klippyState = moonraker.klippyState.state;
  let klippyStateMessage = moonraker.klippyState.message;
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
              <td class="pr-2 text-end text-neutral-100">Klippy</td>
              <td class="text-purple-600">Disconnected</td>
            </tr>
          {:else if $klippyState === 'startup'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klippy</td>
              <td class="text-yellow-600">Startup</td>
            </tr>
          {:else if $klippyState === 'ready'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klippy</td>
              <td class="text-green-600">Ready</td>
            </tr>
          {:else if $klippyState === 'shutdown'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klippy</td>
              <td class="text-orange-600">Shutdown</td>
            </tr>
          {:else if $klippyState === 'error'}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Klippy</td>
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
  <div class="flex w-1/6 flex-col justify-around">
    {#if $isConnected}
      {#if $klippyState !== 'ready'}
        <button
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          on:click|preventDefault="{() => commands.printerRestart()}">Restart</button>
        <button
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          on:click|preventDefault="{() => commands.firmwareRestart()}">Firmware Restart</button>
      {:else}
        <a
          href="/printstate"
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          >Printer State</a>
      {/if}
    {:else}
      <button
        class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
        on:click|preventDefault="{async () => {
          await moonraker.disconnect();
          await moonraker.connect($wsUrl);
        }}">Connect</button>
    {/if}
  </div>
</div>
