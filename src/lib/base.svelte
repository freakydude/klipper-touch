<script lang="ts" module>
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';
  import { Commands } from './commons/Commands';
  import { Values } from './commons/Values';
  import { BootParams } from './commons/BootParams';

  export const bootParams = new BootParams();
  export const client = new JsonRpcClient();
  export const moonraker = new MoonrakerClient(client);
  export const commands = new Commands(client);
  export const values = new Values(moonraker);
</script>
