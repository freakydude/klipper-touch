<script lang="ts">
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { onMount } from 'svelte';
  import '../app.css';

  onMount(async () => {
    await moonraker.connect();
    //console.log(await moonraker.requestIdentifyConnection());

    // client.addEventListener('notification', (event) => {
    //   console.log(event.detail);
    // });
    // let serverInfoRequest = new JsonRpcRequest({ method: 'server.info' });
    // await client.sendRequest(serverInfoRequest);

    // let validRequest = new JsonRpcRequest({ method: 'printer.query_endstops.status' });

    // let errorRequest = new JsonRpcRequest({ method: 'printer.query_endXstops.status' });

    // let batchRequest = [validRequest, errorRequest];
    // console.log('batchRequest', batchRequest);
    // console.log(await client.sendBatchRequest(batchRequest));

    // let objectListRequest = new JsonRpcRequest({ method: 'printer.objects.list' });
    // await client.sendRequest(objectListRequest);
    // let objectQueryRequest = new JsonRpcRequest({
    //   method: 'printer.objects.query',
    //   params: {
    //     objects: {
    //       webhooks: null,
    //       gcode_move: null,
    //       toolhead: null,
    //       extruder: null,
    //       heaters: null,
    //       heater_bed: null,
    //       probe: null,
    //       print_stats: null,
    //       virtual_sdcard: null,
    //       motion_report: null,
    //       mcu: null
    //     }
    //   }
    // });
    // await client.sendRequest(objectQueryRequest);

    // let objectQueryProbeRequest = new JsonRpcRequest({
    //   method: 'printer.objects.query',
    //   params: {
    //     objects: {
    //       probe: ['last_z_result']
    //     }
    //   }
    // });
    // await client.sendRequest(objectQueryProbeRequest);

    let objectSubscribeBedTempRequest = new JsonRpcRequest({
      method: 'printer.objects.subscribe',
      params: {
        objects: {
          heater_bed: ['temperature', 'target'],
          extruder: ['temperature', 'target'],
          toolhead: ['position', 'homed_axes'],
          gcode_move: ['homing_origin']
        }
      }
    });
    await client.sendRequest(objectSubscribeBedTempRequest);

    // Replace Subscription
    //
    // let objectUnSubscribeBedTempRequest = new JsonRpcRequest({
    //   method: 'printer.objects.subscribe',
    //   params: {
    //     objects: {
    //       extruder: ['temperature']
    //     }
    //   }
    // });
    // await client.sendRequest(objectUnSubscribeBedTempRequest);
  });
</script>

<div class="grid h-screen w-screen">
  <slot />
</div>
