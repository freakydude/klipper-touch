<script lang="ts">
  import '../app.css';

  let extruderTemp = 210;
  let heaterBedTemp = 65;
  let printProgress = 42;
  let eta = '20:11';
</script>

<div class="flex justify-start bg-sky-800 ">
  <a class="bg-sky-400 px-3 text-rose-800 " href="/">&triangleleft;</a>
  <div class="flex flex-grow justify-evenly bg-sky-800 px-2">
    <p>E: {extruderTemp} °C</p>
    <p>H: {heaterBedTemp} °C</p>
    <p>P: {printProgress} %</p>
    <p>ETA: {eta}</p>
  </div>
</div>
