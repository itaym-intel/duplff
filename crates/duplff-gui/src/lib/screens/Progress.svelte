<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { currentScreen, report } from '$lib/stores';
  import { onScanProgress, onHashProgress, onScanComplete, onScanError } from '$lib/api';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let phase: 'scanning' | 'hashing' = 'scanning';
  let filesFound = 0;
  let hashDone = 0;
  let hashTotal = 0;
  let elapsed = 0;
  let error: string | null = null;
  let timer: ReturnType<typeof setInterval>;
  let unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    const startTime = Date.now();
    timer = setInterval(() => {
      elapsed = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);

    unlisteners.push(
      await onScanProgress((count) => { filesFound = count; }),
      await onHashProgress((progress) => {
        phase = 'hashing';
        hashDone = progress.done;
        hashTotal = progress.total;
      }),
      await onScanComplete((r) => {
        report.set(r);
        currentScreen.set('results');
      }),
      await onScanError((msg) => { error = msg; }),
    );
  });

  onDestroy(() => {
    clearInterval(timer);
    unlisteners.forEach(fn => fn());
  });

  function handleCancel() {
    currentScreen.set('setup');
  }

  function formatTime(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  $: progressPct = hashTotal > 0 ? Math.round((hashDone / hashTotal) * 100) : 0;
</script>

<div class="max-w-lg mx-auto p-8 mt-20">
  <h2 class="text-2xl font-bold mb-6">
    {#if error}
      Scan failed
    {:else if phase === 'scanning'}
      Scanning files...
    {:else}
      Hashing files...
    {/if}
  </h2>

  {#if error}
    <div class="bg-red-900/30 border border-delete rounded-lg p-4 mb-6">
      <p class="text-delete">{error}</p>
    </div>
    <button on:click={handleCancel} class="w-full bg-gray-700 hover:bg-gray-600 text-white py-2 rounded-lg">
      Back to setup
    </button>
  {:else}
    <div class="w-full bg-gray-800 rounded-full h-3 mb-6 overflow-hidden">
      {#if phase === 'hashing' && hashTotal > 0}
        <div class="bg-active h-full rounded-full transition-all duration-300" style="width: {progressPct}%"></div>
      {:else}
        <div class="bg-active h-full rounded-full animate-pulse w-full opacity-30"></div>
      {/if}
    </div>

    <div class="space-y-2 text-sm text-gray-400 mb-8">
      <div class="flex justify-between">
        <span>Files found</span>
        <span class="text-gray-200 font-mono">{filesFound.toLocaleString()}</span>
      </div>
      {#if phase === 'hashing'}
        <div class="flex justify-between">
          <span>Files hashed</span>
          <span class="text-gray-200 font-mono">{hashDone.toLocaleString()} / {hashTotal.toLocaleString()}</span>
        </div>
        <div class="flex justify-between">
          <span>Progress</span>
          <span class="text-gray-200 font-mono">{progressPct}%</span>
        </div>
      {/if}
      <div class="flex justify-between">
        <span>Elapsed</span>
        <span class="text-gray-200 font-mono">{formatTime(elapsed)}</span>
      </div>
    </div>

    <button on:click={handleCancel} class="w-full bg-gray-700 hover:bg-gray-600 text-white py-2 rounded-lg">
      Cancel
    </button>
  {/if}
</div>
