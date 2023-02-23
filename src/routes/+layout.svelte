<script lang="ts">
  import { moonraker } from '$lib/base.svelte';
  import '../app.css';

  // define initial component state

  let isFullscreen = false;
  let outerElement: Element;

  let isConnected = moonraker.isConnected;
  let klippyState = moonraker.klippyState;
  // let klippyStateMessage = moonraker.klippyStateMessage;

  async function switchFullscreen() {
    if (isFullscreen) {
      await document.exitFullscreen();
      isFullscreen = false;
    } else {
      await outerElement.requestFullscreen();
      isFullscreen = true;
    }
  }

  async function connectToMoonraker() {
    await moonraker.disconnect();
    await moonraker.connect();
  }
</script>

<div class="flex h-screen w-screen" bind:this={outerElement}>
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
          <button class="btn-touch " on:click={async () => switchFullscreen()}>Fullscreen</button>
        </div>
      </div>
      <div class="flex flex-col items-center justify-center gap-2">
        <p class="px-2 text-5xl font-bold text-white">Klipper Touch</p>
        <p class="p-2 text-2xl font-bold text-red-600">by freakyDude</p>
      </div>
    </div>
  {/if}
</div>
