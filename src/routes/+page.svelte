<script lang="ts">
    import { onMount } from 'svelte';
  // $lib auto-resolves to ./src/lib in Svelte.
  import { state, connect } from '$lib/state';

  onMount(async () => {
    connect("192.168.40.6/websocket");
  });

  
</script>

<div class="flex h-full flex-wrap content-center justify-evenly bg-lime-700">
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/extruder"><button>Extruder</button></a>
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/control"><button>Control</button></a>
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/files"><button>Files</button></a>
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/heatbed"><button>Heatbed</button></a>
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/maschine"><button>Maschine</button></a>
  <a class="m-2 basis-1/3 rounded bg-slate-400 p-2" href="/progress"><button>Progress</button></a>
</div>
