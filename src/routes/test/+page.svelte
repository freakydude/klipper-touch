<script>
import { goto } from '$app/navigation';
</script>

<div class="page-dark flex-col items-stretch justify-between gap-1">
  <div class="flex h-1/4 flex-row justify-between gap-1 bg-red-700">
    <button class="btn-touch1 w-24 border-gray-400" on:click="{() => goto('/')}">Tool Head</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Heater Bed</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Extruder</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Fan Speed</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Flow Factor</button>
  </div>
  <div class="flex h-1/4 flex-row justify-between gap-1">
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Speed Factor</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Temp erature</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Bed Level</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
  </div>
  <div class="flex h-1/4 flex-row justify-between gap-1 bg-orange-700">
    <button class="label1 w-1/5" on:click="{() => goto('/')}">Label 12 </button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
  </div>
  <div class="flex h-1/4 flex-row justify-between gap-1 bg-purple-700">
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
    <button class="btn-touch1 w-1/5 border-gray-400" on:click="{() => goto('/')}">Root</button>
  </div>
</div>
