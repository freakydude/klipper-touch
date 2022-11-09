<script lang="ts">
  import { currentScreen } from '../store';
</script>

<div>
  <h1 class="flex  p-6 text-white bg-slate-600">Overview</h1>
  <a class="bg-slate-400" href="/extruder"><button>Extruder</button></a>
  <a class="bg-slate-400" href="/control"><button>Control</button></a>
  <a class="bg-slate-400" href="/files"><button>Files</button></a>
  <a class="bg-slate-400" href="/heatbed"><button>Heatbed</button></a>
  <a class="bg-slate-400" href="/maschine"><button>Maschine</button></a>
  <a class="bg-slate-400" href="/progress"><button>Progress</button></a>
</div>
