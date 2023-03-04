<script lang="ts">
  import { goto } from '$app/navigation';
  import { client } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa';

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }
</script>

<div class="flex flex-grow flex-row gap-1 bg-neutral-800 p-1">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <div class="grow" />
    <button class="btn-touch flex flex-col bg-yellow-600" on:click={() => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex flex-grow flex-col flex-wrap content-center items-center justify-center gap-6 rounded">
    <p class="label text-4xl">Parameters</p>
    <div class="flex flex-row flex-wrap items-center gap-2">
      <button class="btn-touch " on:click={() => goto('/parameter/zoffset')}>Z Offset</button>
      <button class="btn-touch " on:click={() => goto('/parameter/temperatures')}>Temperatures</button>
    </div>
  </div>
</div>
