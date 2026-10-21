<script lang="ts">
  import { bootParams, client, commands, moonraker } from '$lib/base.svelte';
  import Icon from '$lib/icons/Icon.svelte';
  import { klippyStateIcon } from '$lib/icons/stateIcons';

  let wsUrl = bootParams.moonrakerWs;
  let apiUrl = bootParams.moonrakerApi;
  let isConnected = client.isConnected;
  let klippyState = moonraker.webhooks.state;
  let klippyStateMessage = moonraker.webhooks.state_message;
</script>

<div class="flex grow content-between items-stretch justify-around gap-3 overflow-hidden bg-neutral-800">
  <p class="fixed right-1 bottom-1 text-sm text-neutral-100">v0.1.0 experimental</p>

  <div class="flex grow flex-col justify-around p-2">
    <span class="flex flex-col items-center">
      <h1 class="text-center text-4xl font-bold text-neutral-100">Klipper-Touch</h1>
      <h1 class="text-center text-xl text-red-600">by freakyDude</h1>
    </span>
    <span class="flex flex-col items-center p-2">
      <table class="table-auto">
        <tbody>
          {#if $isConnected}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Moonraker</td>
              <td class="text-green-600"><span class="inline-flex items-center gap-1"><Icon name="system-connected" class="h-5 w-5" />Connected</span></td>
            </tr>
            {#if $klippyState === 'disconnected'}
              <tr>
                <td class="pr-2 text-end text-neutral-100">Klippy</td>
                <td class="text-purple-600"><span class="inline-flex items-center gap-1"><Icon name={klippyStateIcon($klippyState)} class="h-5 w-5" />Disconnected</span></td>
              </tr>
            {:else if $klippyState === 'startup'}
              <tr>
                <td class="pr-2 text-end text-neutral-100">Klippy</td>
                <td class="text-yellow-600"><span class="inline-flex items-center gap-1"><Icon name={klippyStateIcon($klippyState)} class="h-5 w-5" />Startup</span></td>
              </tr>
            {:else if $klippyState === 'ready'}
              <tr>
                <td class="pr-2 text-end text-neutral-100">Klippy</td>
                <td class="text-green-600"><span class="inline-flex items-center gap-1"><Icon name={klippyStateIcon($klippyState)} class="h-5 w-5" />Ready</span></td>
              </tr>
            {:else if $klippyState === 'shutdown'}
              <tr>
                <td class="pr-2 text-end text-neutral-100">Klippy</td>
                <td class="text-orange-600"><span class="inline-flex items-center gap-1"><Icon name={klippyStateIcon($klippyState)} class="h-5 w-5" />Shutdown</span></td>
              </tr>
            {:else if $klippyState === 'error'}
              <tr>
                <td class="pr-2 text-end text-neutral-100">Klippy</td>
                <td class="text-red-600"><span class="inline-flex items-center gap-1"><Icon name={klippyStateIcon($klippyState)} class="h-5 w-5" />Error</span></td>
              </tr>
            {/if}
          {:else}
            <tr>
              <td class="pr-2 text-end text-neutral-100">Moonraker</td>
              <td class="text-purple-600">Connecting</td>
            </tr>
            <tr>
              <td class="pr-2 text-end text-neutral-100">Api</td>
              <td class="text-neutral-400">{$apiUrl}</td>
            </tr>
            <tr>
              <td class="pr-2 text-end text-neutral-100">Websocket</td>
              <td class="text-neutral-400">{$wsUrl}</td>
            </tr>
          {/if}
        </tbody>
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
          class="flex h-14 flex-col items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 text-xs font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          onclick={() => commands.printerRestart()}><Icon name="system-restart" />Restart</button
        >
        <button
          class="flex h-14 flex-col items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 text-xs font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          onclick={() => commands.firmwareRestart()}><Icon name="system-firmware-restart" />Firmware</button
        >
      {:else}
        <a
          href="/printstate"
          class="flex h-14 items-center justify-center rounded-l-lg bg-neutral-700 px-3 py-2 font-semibold text-neutral-50 drop-shadow-md active:bg-red-500"
          ><Icon name="nav-printer" label="Printer State" class="h-8 w-8" /></a
        >
      {/if}
    {/if}
  </div>
</div>
