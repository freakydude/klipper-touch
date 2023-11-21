<script lang="ts" context="module">
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';
  import { Commands } from './commons/Commands';
  import { Values } from './commons/Values';
  import { getMatches } from '@tauri-apps/api/cli';
  import { onMount } from 'svelte';

  // onMount(async () => {
  //   let matches = await getMatches();
  //   console.log(matches);
  // });

  export let client: JsonRpcClient = new JsonRpcClient('ws://127.0.0.1/websocket');
  export let moonraker: MoonrakerClient = new MoonrakerClient(client);
  export let commands: Commands = new Commands(client);
  export let values: Values = new Values(moonraker);
</script>
