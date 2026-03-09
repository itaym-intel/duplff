<script lang="ts">
  import { currentScreen } from '$lib/stores';
  import Setup from '$lib/screens/Setup.svelte';
  import Progress from '$lib/screens/Progress.svelte';
  import Results from '$lib/screens/Results.svelte';
</script>

<main class="min-h-screen">
  {#if $currentScreen === 'setup'}
    <Setup />
  {:else if $currentScreen === 'progress'}
    <Progress />
  {:else if $currentScreen === 'results'}
    <Results />
  {/if}
</main>
