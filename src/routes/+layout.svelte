<script lang="ts">
  import { moonraker } from '$lib/base.svelte';
  import '../app.css';

  // define initial component state
  let isFull = false;
  let fsContainer: any = null;

  let isConnected = moonraker.isConnected;
  let klippyState = moonraker.klippyState;
  // let klippyStateMessage = moonraker.klippyStateMessage;

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

  function switchFullscreen() {
    if (fullscreenSupport()) {
      if (!isFull) {
        requestFullscreen();
      } else {
        exitFullscreen();
      }
    }
  }

  async function connectToMoonraker() {
    await moonraker.disconnect();
    await moonraker.connect();
  }
</script>

<div class="flex h-screen w-screen" bind:this={fsContainer}>
  {#if $isConnected && $klippyState == 'ready'}
    <slot />
  {:else}
    <div class="flex grow flex-col flex-wrap place-content-center items-center gap-6 bg-neutral-800">
      <div class="flex flex-col rounded bg-neutral-600">
        <div class="flex flex-col flex-wrap items-stretch gap-1">
          <p class="label-head">Klipper</p>
          <p class="label">State: {$klippyState}</p>
          <!-- <p class="label">Message: {$klippyStateMessage}</p> -->
          <button class="btn-touch " on:click={async () => connectToMoonraker()}>Connect</button>
          <button class="btn-touch " on:click={() => switchFullscreen()}>Fullscreen</button>
        </div>
      </div>
      <div class="flex flex-col items-center justify-center gap-2">
        <p class="px-2 text-5xl font-bold text-white">Klipper Touch</p>
        <p class="p-2 text-2xl font-bold text-red-600">by freakyDude</p>
      </div>
    </div>
  {/if}
</div>
