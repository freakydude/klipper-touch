<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import NavBar from '$lib/NavBar.svelte';
  import StatusBar from '$lib/StatusBar.svelte';
  import { onMount } from 'svelte';
  import '../app.css';
  import MessageBar from '../lib/MessageBar.svelte';

  onMount(async () => {
    await moonraker.connect();
    //console.log(await moonraker.requestIdentifyConnection());

    // client.addEventListener('notification', (event) => {
    //   console.log(event.detail);
    // });
    let serverInfoRequest = new JsonRpcRequest({ method: 'server.info' });
    await client.sendRequest(serverInfoRequest);

    let validRequest = new JsonRpcRequest({ method: 'printer.query_endstops.status' });

    let errorRequest = new JsonRpcRequest({ method: 'printer.query_endXstops.status' });

    let batchRequest = [validRequest, errorRequest];
    console.log('batchRequest', batchRequest);
    console.log(await client.sendBatchRequest(batchRequest));

    let objectListRequest = new JsonRpcRequest({ method: 'printer.objects.list' });
    await client.sendRequest(objectListRequest);
    let objectQueryRequest = new JsonRpcRequest({
      method: 'printer.objects.query',
      params: {
        objects: {
          webhooks: null,
          gcode_move: null,
          toolhead: null,
          extruder: null,
          heaters: null,
          heater_bed: null,
          probe: null,
          print_stats: null,
          virtual_sdcard: null,
          motion_report: null,
          mcu: null
        }
      }
    });
    await client.sendRequest(objectQueryRequest);

    let objectQueryProbeRequest = new JsonRpcRequest({
      method: 'printer.objects.query',
      params: {
        objects: {
          probe: ['last_z_result']
        }
      }
    });
    await client.sendRequest(objectQueryProbeRequest);

    let objectSubscribeBedTempRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: {
        objects: {
          heater_bed: ['temperature'],
          extruder: ['temperature']
        }
      }
    });
    await client.sendRequest(objectSubscribeBedTempRequest);
    // let clearSubRequest = new JsonRpcRequest('printer.objects.subscribe', JsonRpcClient.generateConnectionId(), {
    //   objects: {}
    // });
    // await client.sendRequest(clearSubRequest);
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
