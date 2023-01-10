<script lang="ts">
  import NavBar from './../lib/NavBar.svelte';
  import '../app.css';

  import StatusBar from '../lib/StatusBar.svelte';
  import MessageBar from '../lib/MessageBar.svelte';
  import { onMount } from 'svelte';
  import { MoonrakerConnect } from '$lib/MoonrakerConnect';

  let mc: MoonrakerConnect;

  onMount(async () => {
    mc = new MoonrakerConnect(import.meta.env.VITE_MOONRAKER_WEBSOCKET);

    console.log(await mc.connect());

    const request = {
      jsonrpc: '2.0',
      method: 'printer.query_endstops.status',
      id: mc.generateConnectionId()
    };

    console.log(await mc.sendMessage(request));
  });
</script>

<div class="flex h-screen w-screen flex-col justify-between bg-white dark:bg-black">
  <StatusBar />
  <div class="flex grow flex-row bg-neutral-800">
    <NavBar />
    <slot />
  </div>
  <MessageBar />
</div>
