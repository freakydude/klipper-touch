<script lang="ts">
  async function runGCode(gcode: string) {
    console.log("runGCode:",gcode);

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

  async function calibrateG28() {
    console.log("run G28");

    await runGCode("G28");
    
  }
</script>

<div>
  <h1 class="flex flex-grow justify-center p-6 text-white">Calibrate</h1>
  <button class="bg-yellow-400" on:click={() => runGCode("G28")}
    >RPC Request</button
  >
</div>
