<script lang="ts">
  import '../app.css';
  import { moonraker } from './base.svelte';

  let statusMessage = 'g-Code messages (M117)';

  let red = moonraker.isReady;
</script>

<div class="flex flex-row justify-center bg-sky-800 py-0.5 px-2">
  <p>isReady: {$red}</p>
</div>
