<script lang="ts">
  import '../app.css';
  import { goto } from '$app/navigation';
  import { bootParams, client, moonraker, values } from '$lib/base.svelte';
  import { onMount } from 'svelte';
  import { getMatches } from '@tauri-apps/api/cli';
  import { appWindow } from '@tauri-apps/api/window';

  onMount(async () => {
    let matches = await getMatches();
    bootParams.setMatches(matches);
  });

  let isConnected = client.isConnected;
  let klippyState = moonraker.klippyState.state;
  let isFullscreen = bootParams.fullscreen;
  let wsUrl = bootParams.moonrakerWs;
  let apiUrl = bootParams.moonrakerApi;
  let printFilename = moonraker.printStats.Filename;
  let interval: NodeJS.Timeout;

  $: {
    if ($isConnected) {
      clearInterval(interval);
    } else {
      clearInterval(interval);
      interval = setInterval(async () => {
        //await moonraker.disconnect();
        await moonraker.connect($wsUrl);
      }, 5000);
    }

    if ($isConnected === false || $klippyState !== 'ready') {
      goto('/');
    }
    if ($isConnected === true && $klippyState === 'ready') {
      goto('/printstate');
    }
  }

  $: {
    appWindow.setFullscreen($isFullscreen);
  }

  async function updateMetadata(relativeFilename: string) {
    // console.log('loadedFile', relativeFilename);
    if (relativeFilename !== '') {
      let fileMeta = await values.getFileMetadata(relativeFilename);
      // console.log('fileMeta', fileMeta);
      values.fileMetadata.set(fileMeta);

      if (fileMeta !== null) {
        values.largestAbsoluteThumbnailPath.set(await values.getLargestAbsoluteThumbnailPath($apiUrl, fileMeta.thumbnails));
      }
    }
  }

  $: updateMetadata($printFilename);
</script>

<div class="flex h-screen w-screen">
  <slot />
</div>
