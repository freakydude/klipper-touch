<script lang="ts">
  import NavBar from './../lib/NavBar.svelte';
  import '../app.css';

  import StatusBar from '../lib/StatusBar.svelte';
  import MessageBar from '../lib/MessageBar.svelte';
  import { onMount } from 'svelte';
    import { connect, generateConnectionId, sendMessage } from '$lib/MoonrakerConnect';

  

  onMount(async () => {
    console.log(await connect());

    const request2 = {
      jsonrpc: '2.0',
      method: 'printer.query_endstops.status',
      id: generateConnectionId()
    };

    console.log(await sendMessage(request2));
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
