<script lang="ts">
  import { writable } from 'svelte/store';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import '../app.css';

  // define initial component state
  let isFull = false;
  let fsContainer: any = null;

  let isConnected = moonraker.isConnected;

  // boring plain js fullscreen support stuff below
  const noop = () => {};

  function fullscreenSupport(): boolean {
    return !!(document.fullscreenEnabled || document.webkitFullscreenEnabled || document.mozFullScreenEnabled || document.msFullscreenEnabled || false);
  }

  function exitFullscreen(): void {
    (document.exitFullscreen || document.mozCancelFullScreen || document.webkitExitFullscreen || document.msExitFullscreen || noop).bind(document);
    isFull = false;
  }

  function requestFullscreen(): void {
    const requestFS = (
      fsContainer.requestFullscreen ||
      fsContainer.mozRequestFullScreen ||
      fsContainer.webkitRequestFullscreen ||
      fsContainer.msRequestFullscreen ||
      noop
    ).bind(fsContainer);
    requestFS();
    isFull = true;
  }

  async function connectToMoonraker() {
    // if (fullscreenSupport()) {
    //   requestFullscreen();
    // }

    await moonraker.connect();

    if (isConnected) {
      let objectSubscribeBedTempRequest = new JsonRpcRequest({
        method: 'printer.objects.subscribe',
        params: {
          objects: {
            heater_bed: ['temperature', 'target'],
            extruder: ['temperature', 'target'],
            toolhead: ['position', 'homed_axes'],
            gcode_move: ['homing_origin']
          }
        }
      });

      await client.sendRequest(objectSubscribeBedTempRequest);
    }
  }
</script>

<div class="grid h-screen w-screen">
  <!-- bind:this={fsContainer}> -->
  {#if $isConnected}
    <slot />
  {:else}
    <div class="flex grow flex-col flex-wrap place-content-around items-center bg-neutral-800">
      <button class="btn-touch" on:click={async () => connectToMoonraker()}>Connect</button>
      <div class="flex flex-col flex-wrap items-center justify-center gap-2">
        <p class="px-2 text-5xl font-bold text-white">Klipper Touch</p>
        <p class="p-2 text-2xl font-bold text-red-600">by freakyDude</p>
      </div>
    </div>
  {/if}
</div>
