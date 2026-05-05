<script lang="ts">
  import '../app.css';
  import { goto } from '$app/navigation';
  import { bootParams, client, moonraker } from '$lib/base.svelte';
  import { onDestroy, onMount } from 'svelte';
  import { getMatches } from '@tauri-apps/plugin-cli';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  interface Props {
    children?: import('svelte').Snippet;
  }

  let { children }: Props = $props();

  let unlistenResize: UnlistenFn;
  let unlistenScale: UnlistenFn;

  onMount(async () => {
    const appWindow = getCurrentWindow();

    const scaleUI = async () => {
      const domRect = document.documentElement.getBoundingClientRect();

      const baseFontSize = (domRect.height / 320) * 16;
      document.documentElement.style.fontSize = `${baseFontSize}px`;
    };

    unlistenResize = await appWindow.onResized(async () => {
      await scaleUI();
    });

    unlistenScale = await appWindow.onScaleChanged(async () => {
      await scaleUI();
    });

    await scaleUI();

    await bootParams.loadStore();
    bootParams.setMatches(await getMatches());
  });

  onDestroy(() => {
    if (unlistenResize) {
      unlistenResize();
    }
    if (unlistenScale) {
      unlistenScale();
    }
  });

  let isConnected = client.isConnected;
  let klippyState = moonraker.webhooks.state;
  let isFullscreen = bootParams.fullscreen;
  let wsUrl = bootParams.moonrakerWs;

  $effect(() => {
    if ($isConnected) {
      return;
    }

    const reconnectTimeout = setTimeout(() => {
      void moonraker.connect($wsUrl);
    }, 5000);

    return () => {
      clearTimeout(reconnectTimeout);
    };
  });

  $effect(() => {
    if ($isConnected === false || $klippyState !== 'ready') {
      void goto('/');
    }

    if ($isConnected === true && $klippyState === 'ready') {
      void goto('/printstate');
    }
  });

  $effect(() => {
    getCurrentWindow().setFullscreen($isFullscreen);
  });
</script>

<!-- <style lang="postcss">
  @reference "tailwindcss";
</style> -->
<div class="flex h-screen w-screen">{@render children?.()}</div>
