<script lang="ts" context="module">
  import { JsonRpcClient } from './JsonRpcClient';
  import { MoonrakerRpcClient } from './MoonrakerRpcClient';

  export let client: JsonRpcClient = new JsonRpcClient(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  export let moonraker: MoonrakerRpcClient = new MoonrakerRpcClient(client);
</script>
