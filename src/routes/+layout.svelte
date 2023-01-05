<script lang="ts">
  import '../app.css';

  import StatusBar from '../lib/StatusBar.svelte';
  import MessageBar from '../lib/MessageBar.svelte';
</script>

<div class="flex h-screen w-screen grow flex-col justify-between bg-white dark:bg-black">
  <StatusBar />
  <slot />
  <MessageBar />
</div>
