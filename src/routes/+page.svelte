<script>
import { goto } from '$app/navigation';
</script>

<div class="flex flex-grow flex-col items-center justify-around gap-4 bg-neutral-800">
  <p class="text-5xl font-bold text-gray-300">Klipper Touch</p>
  <button class="btn-touch1 border-green-400" on:click="{() => goto('/printstate')}">Status</button>
  <button class="btn-touch1 border-rose-400" on:click="{() => goto('/test')}">Test</button>
  <p class="text-2xl font-bold text-red-700">by freakyDude</p>
</div>
