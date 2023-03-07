<script lang="ts" context="module">
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';

  export let client: JsonRpcClient = new JsonRpcClient(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  export let moonraker: MoonrakerClient = new MoonrakerClient(client);
</script>
