<script lang="ts">
  import { currentScreen, report, scanConfig, ignoredGroups, keepOverrides, markedFiles, expandedGroups, focusedGroup, filterText } from '$lib/stores';
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

    // Reset all results state for fresh scan
    ignoredGroups.set(new Set());
    keepOverrides.set(new Map());
    markedFiles.set(new Set());
    expandedGroups.set(new Set());
    focusedGroup.set(0);
    filterText.set('');

    const startTime = Date.now();
    timer = setInterval(() => {
      elapsed = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);

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
  <div class="w-full max-w-xs text-center">
    {#if error}
      <div class="bg-delete/8 border border-delete/20 rounded-xl p-5 mb-6">
        <p class="text-delete text-sm">{error}</p>
      </div>
      <button onclick={handleCancel} class="text-sm text-text-muted hover:text-text-secondary transition-colors">
        Back to setup
      </button>
    {:else}
      <!-- Animated scan icon -->
      <div class="mb-8 flex justify-center">
        <div class="w-12 h-12 rounded-xl bg-active/10 border border-active/20 flex items-center justify-center animate-[breathe_2s_ease-in-out_infinite]">
          <span class="text-active text-lg font-mono font-bold">
            {phase === 'scanning' ? '/' : '#'}
          </span>
        </div>
      </div>

      <p class="text-xs font-medium text-text-muted uppercase tracking-[0.2em] mb-8">
        {phase === 'scanning' ? 'Scanning files' : 'Computing hashes'}
      </p>

      <!-- Progress bar -->
      <div class="w-full bg-surface rounded-full h-1 mb-10 overflow-hidden">
        {#if phase === 'hashing' && hashTotal > 0}
          <div class="bg-active h-full rounded-full transition-[width] duration-700 ease-out" style="width: {progressPct}%"></div>
        {:else}
          <div class="h-full rounded-full w-full animate-[shimmer_2s_ease-in-out_infinite]"
            style="background: linear-gradient(90deg, transparent 0%, oklch(0.650 0.170 230.0 / 0.4) 50%, transparent 100%); background-size: 200% 100%;"></div>
        {/if}
      </div>

      <!-- Stats -->
      <div class="space-y-3 text-sm mb-12">
        <div class="flex justify-between items-baseline">
          <span class="text-text-muted text-xs">Files found</span>
          <span class="font-mono text-text-primary tabular-nums">{filesFound.toLocaleString()}</span>
        </div>
        {#if phase === 'hashing'}
          <div class="flex justify-between items-baseline">
            <span class="text-text-muted text-xs">Hashed</span>
            <span class="font-mono text-text-primary tabular-nums">{hashDone.toLocaleString()} <span class="text-text-muted">/</span> {hashTotal.toLocaleString()}</span>
          </div>
        {/if}
        <div class="flex justify-between items-baseline">
          <span class="text-text-muted text-xs">Elapsed</span>
          <span class="font-mono text-text-primary tabular-nums">{formatTime(elapsed)}</span>
        </div>
      </div>

      <button onclick={handleCancel}
        class="text-xs text-text-muted hover:text-text-secondary transition-colors uppercase tracking-widest">
        Cancel
      </button>
    {/if}
  </div>
</div>

<style>
  @keyframes breathe {
    0%, 100% { transform: scale(1); opacity: 0.8; }
    50% { transform: scale(1.05); opacity: 1; }
  }
  @keyframes shimmer {
    0% { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }
</style>
