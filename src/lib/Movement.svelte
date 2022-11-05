<script lang="ts">
  async function runGCode(gcode: string) {
    console.log("runGCode:", gcode);

    const options = {
      method: "POST",
      headers: {
        "Content-Type": "application/json;charset=UTF-8",
      },
      body: "",
    };

    let url = new URL("http://192.168.40.6/printer/gcode/script");
    url.searchParams.set("script", gcode);

    await fetch(url, options)
      .then((res) => {
        let result = JSON.stringify(res);
        console.log(result);
      })
      .catch((error) => {
        console.log(error);
      });
  }

  async function moveUp() {
    console.log("moveUp");
    let gCode = "G91 \n G1 Z+10 F6000 \n G90";

    await runGCode(gCode);
  }

  async function moveDown() {
    console.log("moveDown");
    let gCode = "G91 \n G1 Z-10 F6000 \n G90";

    await runGCode(gCode);
  }

  async function moveLeft() {
    console.log("moveLeft");
    let gCode = "G91 \n G1 X-10 F6000 \n G90";
    await runGCode(gCode);
  }

  async function moveRight() {
    console.log("moveRight");
    let gCode = "G91 \n G1 X+10 F6000 \n G90";
    await runGCode(gCode);
  }
</script>

<div>
  <h1 class="flex flex-grow justify-center p-6 text-white">Movement</h1>
  <div class="grid grid-cols-3 grid-rows-3 gap-4 bg-gray-500">
    <button
      type="button"
      class="col-start-2 inline-block rounded bg-blue-600 px-6 py-6 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-blue-700 hover:shadow-lg focus:bg-blue-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-blue-800 active:shadow-lg"
      on:click={moveUp}
    >
      Up
    </button>
    <button
      type="button"
      class="row-start-2 inline-block rounded bg-blue-600 px-6 py-2.5 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-blue-700 hover:shadow-lg focus:bg-blue-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-blue-800 active:shadow-lg"
      on:click={moveLeft}
      >Left
    </button>
    <button
      type="button"
      class="col-start-3 row-start-2 inline-block rounded bg-blue-600 px-6 py-2.5 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-blue-700 hover:shadow-lg focus:bg-blue-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-blue-800 active:shadow-lg"
      on:click={moveRight}
      >Right
    </button>
    <button
      type="button"
      class="col-start-2 row-start-3 inline-block rounded bg-blue-600 px-6 py-2.5 text-xs font-medium uppercase leading-tight text-white shadow-md transition duration-150 ease-in-out hover:bg-blue-700 hover:shadow-lg focus:bg-blue-700 focus:shadow-lg focus:outline-none focus:ring-0 active:bg-blue-800 active:shadow-lg"
      on:click={moveDown}
      >Down
    </button>
  </div>
</div>
