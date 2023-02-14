<script lang="ts">
  import { goto } from '$app/navigation';

  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faSkull } from '@fortawesome/free-solid-svg-icons';
  import { onMount } from 'svelte';
  import Fa from 'svelte-fa/src/fa.svelte';

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }

  let activeFilename = '';
  let availableFiles = [''];
  let fileMeta = {};
  let thumbnail;

  onMount(async () => {
    await listFiles();
    if (activeFilename == '') {
      if (availableFiles.length > 0) {
        selectFile(availableFiles[0]);
      }
    }
  });

  async function listFiles() {
    let listFilesRequest = new JsonRpcRequest({
      method: 'server.files.list',
      params: {}
    });
    let listFilesResponse = await client.sendRequest(listFilesRequest);

    //console.log(listFilesResponse.result.map(file => file.path));
    availableFiles = listFilesResponse.result.map((file) => file.path);
  }

  async function selectFile(fileName: string) {
    let fileMetaRequest = new JsonRpcRequest({
      method: 'server.files.metadata',
      params: {
        filename: fileName
      }
    });
    let fileMetaResponse = await client.sendRequest(fileMetaRequest);

    fileMeta = fileMetaResponse.result;

    // get thumbnail with largest width
    if (Array.isArray(fileMeta.thumbnails) && fileMeta.thumbnails.length > 0) {
      thumbnail = fileMeta.thumbnails.sort((n1, n2) => n1.width - n2.width)[0];
    }

    thumbnail = import.meta.env.VITE_MOONRAKER_API + 'server/files/gcodes/' + fileMeta.thumbnails[2].relative_path;

    // TODO set this only if print is not running
    activeFilename = fileName;
  }
</script>

<div class="flex flex-grow flex-row gap-1 bg-neutral-800 p-1">
  <div class="flex flex-col gap-1">
    <button class="btn-touch bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch bg-red-600" on:click={() => goto('/printerstatus')}>PS</button>
    <div class="grow" />
    <button class="btn-touch bg-yellow-600 " on:click={async () => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex flex-grow flex-row items-center justify-around gap-2">
    <div class="flex h-screen flex-col overflow-y-auto rounded  bg-neutral-600">
      <div class="flex flex-col">
        <p class="label-head">Files</p>
        <div class="flex flex-col gap-1 py-1">
          {#each availableFiles as avFile}
            <button class="btn-list" on:click={async () => selectFile(avFile)}>{avFile.slice(0, -6)}</button>
          {/each}
        </div>
      </div>
    </div>
    <div class="flex w-1/3 flex-col rounded bg-neutral-600">
      <p class="label-head">Details</p>
      <div class="label">File: {activeFilename.slice(0, -6)}</div>
      <img src={thumbnail} alt="" loading="lazy" />
      <div class="label">
        Printduration: {new Date(fileMeta.estimated_time * 1000).getHours()} h {new Date(fileMeta.estimated_time * 1000).getMinutes()} min
      </div>
      <div class="label">NozzleTemp: {fileMeta.first_layer_extr_temp} °C</div>
      <div class="label">BedTemp: {fileMeta.first_layer_bed_temp} °C</div>
      <div class="label">Layerheight: {fileMeta.layer_height}</div>
    </div>
  </div>
</div>
