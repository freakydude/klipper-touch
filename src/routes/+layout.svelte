<script lang="ts">
  import '../app.css';
  import { JsonRpcClient, JsonRpcRequest } from '$lib/JsonRpcClient';
  import MessageBar from '../lib/MessageBar.svelte';
  import { client, moonraker } from '$lib/base.svelte';
  import { onMount } from 'svelte';
  import NavBar from '$lib/NavBar.svelte';
  import StatusBar from '$lib/StatusBar.svelte';

  onMount(async () => {
    await moonraker.connect();
    //console.log(await moonraker.requestIdentifyConnection());

    // client.addEventListener('notification', (event) => {
    //   console.log(event.detail);
    // });
    let serverInfoRequest = new JsonRpcRequest('server.info', JsonRpcClient.generateConnectionId(), undefined);
    await client.sendRequest(serverInfoRequest);

    // let validRequest = new JsonRpcRequest('printer.query_endstops.status', JsonRpcClient.generateConnectionId(), undefined);

    // console.log(await client.sendRequest(validRequest));
    // let errorRequest = new JsonRpcRequest('printer.query_endsXtops.status', JsonRpcClient.generateConnectionId(), undefined);

    // console.log(await client.sendRequest(errorRequest));

    // let batchRequest = [validRequest, errorRequest];
    // console.log('batchRequest', batchRequest);
    // console.log(await client.sendBatchRequest(batchRequest));

    let objectListRequest = new JsonRpcRequest('printer.objects.list', JsonRpcClient.generateConnectionId(), undefined);
    await client.sendRequest(objectListRequest);
    let objectQueryRequest = new JsonRpcRequest('printer.objects.query', JsonRpcClient.generateConnectionId(), {
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
    });
    await client.sendRequest(objectQueryRequest);

    let objectQueryProbeRequest = new JsonRpcRequest('printer.objects.query', JsonRpcClient.generateConnectionId(), {
      objects: {
        probe: ['last_z_result']
      }
    });
    await client.sendRequest(objectQueryProbeRequest);

    let objectSubscribeBedTempRequest = new JsonRpcRequest('printer.objects.subscribe', JsonRpcClient.generateConnectionId(), {
      objects: {
        heater_bed: ['temperature'],
        extruder: ['temperature']
      }
    });
    await client.sendRequest(objectSubscribeBedTempRequest);
    let clearSubRequest = new JsonRpcRequest('printer.objects.subscribe', JsonRpcClient.generateConnectionId(), {
      objects: {}
    });
    await client.sendRequest(clearSubRequest);
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
