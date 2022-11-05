<script lang="ts">
  import Movement from "./lib/Movement.svelte";
  import Calibrate from "./lib/Calibrate.svelte";
  import Printerstate from "./lib/Printerstate.svelte";

  let currentComponent = Movement;

  async function homeAxies() {
    console.log("homeAxies");
    //let url = encodeURIComponent("SET_GCODE_OFFSET Z_ADJUST=+0.01 MOVE=1")
    let url = encodeURIComponent("G28");

    const res = await fetch(
      "http://192.168.40.6/printer/gcode/script?script=" + url
    )
      .then((res) => {
        let result = JSON.stringify(res);
        console.log(result);
      })
      .catch((error) => {
        console.log(error);
      });
  }
</script>

<main class="bg-zinc-800 ">
  <div
    class="grid h-screen w-screen grid-cols-2 grid-rows-3 justify-items-start "
  >
    <button
      class="grid-start-1 row-start-1 self-center rounded bg-red-600 px-6 py-6 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-red-700 hover:shadow-lg focus:bg-red-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-yellow-800 active:shadow-lg"
      on:click={() => {
        currentComponent = Movement;
      }}
    >
      P1
    </button>
    <button
      class="grid-start-1 row-start-2 self-center rounded bg-red-600 px-6 py-6 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-red-700 hover:shadow-lg focus:bg-red-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-yellow-800 active:shadow-lg"
      on:click={() => {
        currentComponent = Printerstate;
      }}
    >
      P2
    </button>
    <button
      class="grid-start-1 row-start-3 self-center rounded bg-red-600 px-6 py-6 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-red-700 hover:shadow-lg focus:bg-red-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-yellow-800 active:shadow-lg"
      on:click={() => {
        currentComponent = Calibrate;
      }}
    >
      P3
    </button>

    <div class="col-start-2 row-span-3 row-start-1 self-center">
      <svelte:component this={currentComponent} />
    </div>
  </div>
</main>
