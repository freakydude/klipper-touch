<script lang="ts">
  import { writable } from 'svelte/store';
  import '../app.css';
  import { moonraker } from './base.svelte';

  let extruderTemp = moonraker.extruderCurrentTemperature;
  let heaterBedTemp = moonraker.heaterBedCurrentTemperature;
  let printProgress = writable(42);
  let eta = writable('20:11');
</script>

<div class="flex flex-row  justify-evenly bg-sky-800 px-2 py-0.5">
  <p>Extruder: {$extruderTemp.toFixed(1)} °C</p>
  <p>Bed: {$heaterBedTemp.toFixed(1)} °C</p>
  <p>Progress: {$printProgress} %</p>
  <p>ETA: {$eta}</p>
</div>
