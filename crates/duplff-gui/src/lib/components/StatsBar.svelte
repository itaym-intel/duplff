<script lang="ts">
  import { report, filterText, sortMode, currentScreen } from '$lib/stores';
  import { formatBytes, formatNumber } from '$lib/format';
  import type { SortMode } from '$lib/types';

  let groups = $derived($report?.groups.length ?? 0);
  let duplicates = $derived($report?.total_duplicates ?? 0);
  let wasted = $derived($report?.total_wasted_bytes ?? 0);

  let {
    onExportJson,
    onExportCsv,
  }: {
    onExportJson: () => void;
    onExportCsv: () => void;
  } = $props();

  let showExport = $state(false);

  function handleSort(e: Event) {
    sortMode.set((e.target as HTMLSelectElement).value as SortMode);
  }

  function newScan() {
    report.set(null);
    currentScreen.set('setup');
  }
</script>

<div class="flex flex-col gap-3 px-5 py-4 border-b border-border bg-surface/80 backdrop-blur-md sticky top-0 z-10">
  <!-- Stats row -->
  <div class="flex items-center gap-5 text-sm">
    <div class="flex items-center gap-2">
      <img src="/logo.svg" alt="duplff" class="w-6 h-6" />
      <span class="text-xs font-medium text-text-muted uppercase tracking-widest">Results</span>
    </div>

    <div class="w-px h-4 bg-border-subtle"></div>

    <div class="flex items-center gap-5">
      <span class="text-text-muted text-xs">
        <span class="font-mono font-medium text-text-primary">{formatNumber(groups)}</span> groups
      </span>
      <span class="text-text-muted text-xs">
        <span class="font-mono font-medium text-text-primary">{formatNumber(duplicates)}</span> duplicates
      </span>
      <span class="text-xs">
        <span class="font-mono font-medium text-delete">{formatBytes(wasted)}</span>
        <span class="text-delete/70">wasted</span>
      </span>
    </div>

    <div class="flex-1"></div>

    <button onclick={newScan}
      class="text-xs text-text-muted hover:text-text-secondary transition-colors uppercase tracking-widest">
      New scan
    </button>
  </div>

  <!-- Toolbar row -->
  <div class="flex items-center gap-2.5">
    <div class="relative flex-1">
      <input
        type="text"
        placeholder="Filter groups..."
        bind:value={$filterText}
        class="w-full bg-surface border border-border rounded-lg px-3 py-2 text-sm text-text-primary
          focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15
          placeholder-text-muted transition-shadow"
      />
      <span class="absolute right-3 top-1/2 -translate-y-1/2 text-[10px] text-text-muted font-mono
        bg-surface-raised border border-border-subtle rounded px-1.5 py-0.5 leading-none">/</span>
    </div>

    <select
      value={$sortMode}
      onchange={handleSort}
      class="bg-surface border border-border rounded-lg px-3 py-2 text-sm text-text-secondary cursor-pointer
        focus:border-active/40 focus:outline-none focus:ring-1 focus:ring-active/15 transition-shadow"
    >
      <option value="wasted">Most wasted</option>
      <option value="size">Largest files</option>
      <option value="files">Most copies</option>
      <option value="path">Path A-Z</option>
    </select>

    <div class="relative">
      <button
        onclick={() => showExport = !showExport}
        class="bg-surface border border-border rounded-lg px-3 py-2 text-sm text-text-secondary
          hover:text-text-primary hover:border-border transition-all"
      >
        Export
      </button>
      {#if showExport}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="fixed inset-0 z-20" onclick={() => showExport = false}></div>
        <div class="absolute right-0 top-full mt-1.5 bg-surface-raised border border-border rounded-xl
          shadow-2xl shadow-black/40 z-30 py-1.5 min-w-32 animate-[fadeIn_100ms_ease-out]">
          <button onclick={() => { onExportJson(); showExport = false; }}
            class="w-full text-left px-3.5 py-2 text-sm text-text-secondary hover:bg-surface-hover
              hover:text-text-primary transition-colors">
            JSON
          </button>
          <button onclick={() => { onExportCsv(); showExport = false; }}
            class="w-full text-left px-3.5 py-2 text-sm text-text-secondary hover:bg-surface-hover
              hover:text-text-primary transition-colors">
            CSV
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
