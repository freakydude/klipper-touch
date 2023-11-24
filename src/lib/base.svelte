<script lang="ts" context="module">
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';
  import { Commands } from './commons/Commands';
  import { Values } from './commons/Values';
  import { BootParams } from './commons/BootParams';

  export let bootParams: BootParams = new BootParams();
  export let client: JsonRpcClient = new JsonRpcClient();
  export let moonraker: MoonrakerClient = new MoonrakerClient(client);
  export let commands: Commands = new Commands(client);
  export let values: Values = new Values(moonraker);
</script>
