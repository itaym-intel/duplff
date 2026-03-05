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

<div class="flex items-center justify-center min-h-screen p-6">
  <div class="w-full max-w-sm text-center">
    {#if error}
      <p class="text-delete text-sm mb-6">{error}</p>
      <button on:click={handleCancel} class="text-sm text-gray-500 hover:text-gray-300 transition-colors">
        Back to setup
      </button>
    {:else}
      <p class="text-xs text-gray-500 uppercase tracking-wide mb-4">
        {phase === 'scanning' ? 'Scanning' : 'Hashing'}
      </p>

      <div class="w-full bg-gray-800 rounded-full h-1 mb-6 overflow-hidden">
        {#if phase === 'hashing' && hashTotal > 0}
          <div class="bg-active h-full rounded-full transition-[width] duration-500 ease-out" style="width: {progressPct}%"></div>
        {:else}
          <div class="bg-active/30 h-full rounded-full w-full animate-[pulse_2s_ease-in-out_infinite]"></div>
        {/if}
      </div>

      <div class="space-y-1 text-sm mb-8">
        <div class="flex justify-between text-gray-500">
          <span>Files found</span>
          <span class="font-mono text-gray-300">{filesFound.toLocaleString()}</span>
        </div>
        {#if phase === 'hashing'}
          <div class="flex justify-between text-gray-500">
            <span>Hashed</span>
            <span class="font-mono text-gray-300">{hashDone.toLocaleString()} / {hashTotal.toLocaleString()}</span>
          </div>
        {/if}
        <div class="flex justify-between text-gray-500">
          <span>Elapsed</span>
          <span class="font-mono text-gray-300">{formatTime(elapsed)}</span>
        </div>
      </div>

      <button on:click={handleCancel} class="text-sm text-gray-500 hover:text-gray-300 transition-colors">
        Cancel
      </button>
    {/if}
  </div>
</div>
