<script lang="ts" context="module">
  import { readable } from 'svelte/store';
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';

  export let client: JsonRpcClient = new JsonRpcClient(import.meta.env.VITE_MOONRAKER_WEBSOCKET);
  export let moonraker: MoonrakerClient = new MoonrakerClient(client);

  export const clockFormatter = new Intl.DateTimeFormat('de', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit'
  });

  export const clock = readable(new Date(), function start(set) {
    const interval = setInterval(() => {
      set(new Date());
    }, 1000);

    return function stop() {
      clearInterval(interval);
    };
  });
</script>
