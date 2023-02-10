<script lang="ts">
  import { goto } from '$app/navigation';
  import { client } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faSkull } from '@fortawesome/free-solid-svg-icons';
  import Fa from 'svelte-fa/src/fa.svelte';

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }
</script>

<div class="flex flex-row bg-neutral-800 p-2">
  <div class="flex flex-col justify-start gap-1">
    <button class="btn-touch flex flex-col bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch flex flex-col bg-yellow-600" on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex grow flex-col flex-wrap place-content-around items-center bg-neutral-800">
    <p class="label text-4xl">Parameters</p>
    <div class="flex flex-row flex-wrap items-center gap-2">
      <button class="btn-touch " on:click={() => goto('/parameter/zoffset')}>Z Offset</button>
      <button class="btn-touch " on:click={() => goto('/parameter/temperatures')}>Temperatures</button>
    </div>
  </div>
</div>
