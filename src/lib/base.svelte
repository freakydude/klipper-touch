<script lang="ts" context="module">
  import { readable } from 'svelte/store';
  import { JsonRpcClient } from './jsonrpc/JsonRpcClient';
  import { MoonrakerClient } from './moonraker/MoonrakerClient';
  import { env } from '$env/dynamic/public';

  export let client: JsonRpcClient = new JsonRpcClient(env.PUBLIC_KT_MOONRAKER_WS === undefined ? 'ws://127.0.0.1/websocket' : env.PUBLIC_KT_MOONRAKER_WS);
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
