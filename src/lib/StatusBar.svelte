<script lang="ts">
  import '../app.css';

  let extruderTemp = 210;
  let heaterBedTemp = 65;
  let printProgress = 42;
  let eta = '20:11';
</script>

<div class="flex flex-row  justify-evenly bg-sky-800 px-2 py-0.5">
  <p>E: {extruderTemp} °C</p>
  <p>H: {heaterBedTemp} °C</p>
  <p>P: {printProgress} %</p>
  <p>ETA: {eta}</p>
</div>
