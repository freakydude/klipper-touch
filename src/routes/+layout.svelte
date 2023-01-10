<script lang="ts">
  import NavBar from './../lib/NavBar.svelte';
  import '../app.css';

  import StatusBar from '../lib/StatusBar.svelte';
  import MessageBar from '../lib/MessageBar.svelte';
  import { onMount } from 'svelte';

  import { MoonrakerRpcClient } from '$lib/MoonrakerRpcClient';
  import { JsonRpcClient, JsonRpcRequest } from '$lib/JsonRpcClient';

  let client: JsonRpcClient;
  let moonraker: MoonrakerRpcClient;

  onMount(async () => {
    client = new JsonRpcClient(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
    moonraker = new MoonrakerRpcClient(client);
    console.log(await client.connect());
    console.log(await moonraker.requestIdentifyConnection());

    let validRequest = new JsonRpcRequest(client.generateConnectionId(), 'printer.query_endstops.status', undefined);

    console.log(await client.sendMessage(validRequest));
    let errorRequest = new JsonRpcRequest(client.generateConnectionId(), 'printer.query_endsXtops.status', undefined);

    console.log(await client.sendMessage(errorRequest));
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
