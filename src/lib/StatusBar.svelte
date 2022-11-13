<script lang="ts">
  import { writable } from 'svelte/store';
  import '../app.css';

  const headerBedTemp = writable<number>();
  const extruderTemp = writable<number>();
  const printProgress = writable<number>();

  async function getStatus() {
    console.log('getStatus query');

    const response = await fetch(
      'http://192.168.40.6/printer/objects/query?heater_bed=temperature&extruder=temperature&display_status=progress'
    );

    return await response.json();
  }

  setInterval(() => {
    getStatus().then((response) => {
      headerBedTemp.set(response.result.status.heater_bed.temperature);
      extruderTemp.set(response.result.status.extruder.temperature);
      printProgress.set(response.result.status.display_status.progress);
    });
  }, 1000);
</script>

<div class="flex justify-start bg-yellow-600 ">
  <a class="rounded bg-red-500 px-2 " href="/">&triangleleft;</a>

  <div class="flex flex-grow justify-evenly bg-yellow-200 px-2">
    <p>E: {$extruderTemp}°C</p>
    <p>H: {$headerBedTemp}°C</p>
    <p>P: {$printProgress}%</p>
    <p>ETA: 20:11</p>
  </div>
</div>
