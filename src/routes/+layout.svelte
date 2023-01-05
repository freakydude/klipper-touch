<script lang="ts">
  import StatusBar from '../lib/StatusBar.svelte';
  import '../app.css';
  import MessageBar from '../lib/MessageBar.svelte';
</script>

<div class=" flex h-screen flex-col justify-between bg-white dark:bg-black">
  <StatusBar />
  <slot />
  <MessageBar />
</div>
