<script lang="ts" context="module">
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';
  import { Commands } from './commons/Commands';
  import { Values } from './commons/Values';
  import { env } from '$env/dynamic/public';

  export let client: JsonRpcClient = new JsonRpcClient(env.PUBLIC_KT_MOONRAKER_WS === undefined ? 'ws://127.0.0.1/websocket' : env.PUBLIC_KT_MOONRAKER_WS);
  export let moonraker: MoonrakerClient = new MoonrakerClient(client);
  export let commands: Commands = new Commands(client);
  export let values: Values = new Values(moonraker);


</script>
