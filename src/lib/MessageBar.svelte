<script lang="ts">
  import { writable } from 'svelte/store';
  import '../app.css';

  const printMessage = writable<string>();

  async function getStatus() {
    console.log('getStatus query');

    const response = await fetch(
      'http://192.168.40.6/printer/objects/query?display_status=message'
    );

    return await response.json();
  }

  setInterval(() => {
    getStatus().then((response) => {
      printMessage.set(response.result.status.display_status.message);
    });
  }, 1000);
</script>

<div class="flex justify-start bg-blue-600">
  <p class="ml-2">{$printMessage}</p>
</div>
