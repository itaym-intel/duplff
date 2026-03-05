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

<div class="flex flex-col gap-2 px-4 py-3 border-b border-border bg-gray-950/80 backdrop-blur-sm sticky top-0 z-10">
  <!-- Stats row -->
  <div class="flex items-center gap-6 text-sm">
    <span class="text-text-secondary">
      <span class="font-mono font-medium text-text-primary">{formatNumber(groups)}</span> groups
    </span>
    <span class="text-text-secondary">
      <span class="font-mono font-medium text-text-primary">{formatNumber(duplicates)}</span> duplicates
    </span>
    <span class="text-delete">
      <span class="font-mono font-medium">{formatBytes(wasted)}</span> wasted
    </span>
    <div class="flex-1"></div>
    <button onclick={newScan} class="text-sm text-text-muted hover:text-text-secondary transition-colors">
      New Scan
    </button>
  </div>

  <!-- Toolbar row -->
  <div class="flex items-center gap-2">
    <div class="relative flex-1">
      <input
        type="text"
        placeholder="Filter groups..."
        bind:value={$filterText}
        class="w-full bg-surface border border-border rounded-lg px-3 py-1.5 text-sm
          focus:border-active/50 focus:outline-none focus:ring-1 focus:ring-active/20
          placeholder-text-muted"
      />
      <span class="absolute right-2.5 top-1/2 -translate-y-1/2 text-xs text-text-muted font-mono">/</span>
    </div>

    <select
      value={$sortMode}
      onchange={handleSort}
      class="bg-surface border border-border rounded-lg px-3 py-1.5 text-sm text-text-secondary cursor-pointer"
    >
      <option value="wasted">Most wasted</option>
      <option value="size">Largest files</option>
      <option value="files">Most copies</option>
      <option value="path">Path A-Z</option>
    </select>

    <div class="relative">
      <button
        onclick={() => showExport = !showExport}
        class="bg-surface border border-border rounded-lg px-3 py-1.5 text-sm text-text-secondary hover:text-text-primary transition-colors"
      >
        Export
      </button>
      {#if showExport}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="fixed inset-0 z-20" onclick={() => showExport = false}></div>
        <div class="absolute right-0 top-full mt-1 bg-gray-900 border border-border rounded-lg shadow-xl z-30 py-1 min-w-28">
          <button onclick={() => { onExportJson(); showExport = false; }}
            class="w-full text-left px-3 py-1.5 text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary transition-colors">
            JSON
          </button>
          <button onclick={() => { onExportCsv(); showExport = false; }}
            class="w-full text-left px-3 py-1.5 text-sm text-text-secondary hover:bg-surface-hover hover:text-text-primary transition-colors">
            CSV
          </button>
        </div>
      {/if}
    </div>
  </div>
</div>
