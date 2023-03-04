<script lang="ts">
  import { goto } from '$app/navigation';
  import { client, moonraker } from '$lib/base.svelte';
  import { JsonRpcRequest } from '$lib/JsonRpcClient';
  import { faList, faPrint, faSkull, faTrash } from '@fortawesome/free-solid-svg-icons';
  import { onMount } from 'svelte';
  import Fa from 'svelte-fa';

  async function emergencyStop() {
    let stopRequest = new JsonRpcRequest({
      method: 'printer.emergency_stop',
      params: {}
    });
    await client.sendRequest(stopRequest);
  }

  let printState = moonraker.printStatsState;

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
    availableFiles = await listFilesResponse.result.sort((n1, n2) => n2.modified - n1.modified).map((file) => file.path);
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

  async function printFile(fileName: string) {
    let printRequest = new JsonRpcRequest({
      method: 'printer.print.start',
      params: { filename: fileName }
    });
    await client.sendRequest(printRequest);
  }

  async function deleteFile(fileName: string) {
    let deleteRequest = new JsonRpcRequest({
      method: 'server.files.delete_file',
      params: { path: 'gcodes/' + fileName }
    });
    await client.sendRequest(deleteRequest);
  }
</script>

<div class="flex flex-grow flex-row gap-1 overflow-hidden bg-neutral-800 p-1">
  <div class="flex flex-col gap-1">
    <button class="btn-touch bg-red-600" on:click={() => goto('/')}><Fa icon={faList} /></button>
    <button class="btn-touch bg-red-600" on:click={() => goto('/printerstatus')}>PS</button>
    <div class="grow" />
    <button class="btn-touch bg-yellow-600 " on:click={() => emergencyStop()}><Fa icon={faSkull} /></button>
  </div>

  <div class="flex flex-col overflow-y-auto overflow-x-hidden rounded bg-neutral-600">
    <div class="flex flex-col">
      <p class="label-head">Files</p>
      <div class="flex flex-col gap-1 py-1">
        {#each availableFiles as avFile}
          <button class="btn-list {avFile == activeFilename ? ' border-2 border-red-600' : ''}" on:click={async () => selectFile(avFile)}
            >{avFile.slice(0, -6)}</button
          >
        {/each}
      </div>
    </div>
  </div>

  <div class="flex flex-col items-center justify-around  rounded">
    <div class="flex flex-col overflow-y-auto rounded bg-neutral-600">
      <p class="label-head">Details</p>
      <!-- <p class="label overflow-x-clip">File: {activeFilename.slice(0, -6)}</p> -->
      <img src={thumbnail} alt="" loading="lazy" class="h-24 self-center" />
      <!--  class="w-24 self-center" -->
      <p class="label py-1">
        {(fileMeta.estimated_time / 60.0).toFixed(0)} min
      </p>
      <p class="label py-1">Nozzle: {fileMeta.first_layer_extr_temp} °C</p>
      <p class="label py-1">Bed: {fileMeta.first_layer_bed_temp} °C</p>
      <p class="label py-1">Layer: {fileMeta.layer_height}</p>
      <div class="flex flex-row justify-around gap-1">
        <button class="btn-touch disabled={$printState != ('printing' || 'paused')}" on:click={() => printFile(activeFilename)}>
          <Fa icon={faPrint} />
        </button>
        <button class="btn-touch" on:click={() => deleteFile(activeFilename)}><Fa icon={faTrash} /></button>
      </div>
    </div>
  </div>
</div>
