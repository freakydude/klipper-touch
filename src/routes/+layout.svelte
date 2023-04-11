<script lang="ts">
  import '../app.css';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/jsonrpc/types/JsonRpcRequest';

  // define initial component state

  let isFullscreen = false;
  let outerElement: Element;

  let klippyState = moonraker.klippyState.state;
  let klippyStateMessage = moonraker.klippyState.message;

  async function switchFullscreen() {
    if (isFullscreen) {
      await document.exitFullscreen();
      isFullscreen = false;
    } else {
      await outerElement.requestFullscreen();
      isFullscreen = true;
    }
  }

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

  // onMount(async () => {
  //   await connectToMoonraker();
  // });
</script>

<div class="flex h-screen w-screen" bind:this={outerElement}>
  {#if $klippyState === 'ready'}
    <slot />
  {:else}
    <div class="flex grow flex-col flex-wrap place-content-center items-center gap-6 bg-neutral-800">
      <div class="flex flex-col rounded bg-neutral-600">
        <div class="flex flex-row flex-wrap justify-center gap-1">
          {#if $klippyState === 'disconnected'}
            <button class="btn-touch" on:click={() => reconnectToMoonraker()}>Reconnect</button>
            <button class="btn-touch" on:click={() => switchFullscreen()}>Fullscreen</button>
          {:else}
            <p class="label">{$klippyStateMessage}</p>
            <button class="btn-touch" on:click={() => printerRestart()}>Restart Printer</button>
            <button class="btn-touch" on:click={() => firmwareRestart()}>Restart Firmware</button>
          {/if}
        </div>
      </div>
      {#if $klippyState === 'disconnected'}
        <div class="flex flex-col items-center justify-center gap-2">
          <p class="px-2 text-5xl font-bold text-white">Klipper Touch</p>
          <p class="p-2 text-2xl font-bold text-red-600">by freakyDude</p>
        </div>
      {/if}
    </div>
  {/if}
</div>
