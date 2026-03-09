<script lang="ts">
  import { currentScreen, report, scanConfig } from '$lib/stores';
  import { startScan, onScanProgress, onHashProgress, onScanComplete, onScanError } from '$lib/api';
  import { get } from 'svelte/store';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  let phase: 'scanning' | 'hashing' = $state('scanning');
  let filesFound = $state(0);
  let hashDone = $state(0);
  let hashTotal = $state(0);
  let elapsed = $state(0);
  let error: string | null = $state(null);

  $effect(() => {
    let timer: ReturnType<typeof setInterval>;
    let unlisteners: UnlistenFn[] = [];
    let cancelled = false;

    const startTime = Date.now();
    timer = setInterval(() => {
      elapsed = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);

    // Register listeners first, then start scan to avoid race condition
    (async () => {
      unlisteners.push(
        await onScanProgress((count) => { if (!cancelled) filesFound = count; }),
        await onHashProgress((progress) => {
          if (cancelled) return;
          phase = 'hashing';
          hashDone = progress.done;
          hashTotal = progress.total;
        }),
        await onScanComplete((r) => {
          if (cancelled) return;
          report.set(r);
          currentScreen.set('results');
        }),
        await onScanError((msg) => { if (!cancelled) error = msg; }),
      );

      // Now that listeners are registered, start the scan
      if (!cancelled) {
        const config = get(scanConfig);
        await startScan(config);
      }
    })();

    return () => {
      cancelled = true;
      clearInterval(timer);
      unlisteners.forEach(fn => fn());
    };
  });

  function handleCancel() {
    currentScreen.set('setup');
  }

  function formatTime(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return m > 0 ? `${m}m ${s}s` : `${s}s`;
  }

  let progressPct = $derived(hashTotal > 0 ? Math.round((hashDone / hashTotal) * 100) : 0);
</script>

<div class="flex items-center justify-center min-h-screen p-8">
  <div class="w-full max-w-sm text-center">
    {#if error}
      <p class="text-delete text-sm mb-6">{error}</p>
      <button onclick={handleCancel} class="text-sm text-text-muted hover:text-text-secondary transition-colors">
        Back to setup
      </button>
    {:else}
      <p class="text-sm text-text-muted uppercase tracking-wider mb-6">
        {phase === 'scanning' ? 'Scanning files' : 'Computing hashes'}
      </p>

      <div class="w-full bg-surface rounded-full h-1.5 mb-8 overflow-hidden">
        {#if phase === 'hashing' && hashTotal > 0}
          <div class="bg-active h-full rounded-full transition-[width] duration-500 ease-out" style="width: {progressPct}%"></div>
        {:else}
          <div class="bg-active/30 h-full rounded-full w-full animate-[pulse_2s_ease-in-out_infinite]"></div>
        {/if}
      </div>

      <div class="space-y-2 text-sm mb-10">
        <div class="flex justify-between text-text-muted">
          <span>Files found</span>
          <span class="font-mono text-text-primary">{filesFound.toLocaleString()}</span>
        </div>
        {#if phase === 'hashing'}
          <div class="flex justify-between text-text-muted">
            <span>Hashed</span>
            <span class="font-mono text-text-primary">{hashDone.toLocaleString()} / {hashTotal.toLocaleString()}</span>
          </div>
        {/if}
        <div class="flex justify-between text-text-muted">
          <span>Elapsed</span>
          <span class="font-mono text-text-primary">{formatTime(elapsed)}</span>
        </div>
      </div>

      <button onclick={handleCancel} class="text-sm text-text-muted hover:text-text-secondary transition-colors">
        Cancel
      </button>
    {/if}
  </div>
</div>
