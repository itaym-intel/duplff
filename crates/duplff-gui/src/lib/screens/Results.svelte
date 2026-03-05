<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles, filterText, sortMode } from '$lib/stores';
  import { trashFiles, exportJson, exportCsv, undoLast, getResults } from '$lib/api';
  import { formatBytes } from '$lib/format';
  import GroupTable from '$lib/components/GroupTable.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import type { SortMode } from '$lib/types';

  let showConfirm = false;
  let toast: { message: string; type: 'info' | 'error' } | null = null;
  let toastTimer: ReturnType<typeof setTimeout>;

  $: groups = $report?.groups ?? [];
  $: totalDuplicates = $report?.total_duplicates ?? 0;
  $: totalWasted = $report?.total_wasted_bytes ?? 0;

  function showToast(message: string, type: 'info' | 'error' = 'info') {
    clearTimeout(toastTimer);
    toast = { message, type };
    toastTimer = setTimeout(() => { toast = null; }, 3000);
  }

  function selectGroup(index: number) {
    selectedGroup.set(index);
    currentScreen.set('detail');
  }

  function autoSelectAll() {
    const paths = new Set<string>();
    for (const group of groups) {
      for (const dup of group.duplicates) {
        paths.add(dup.entry.path);
      }
    }
    markedFiles.set(paths);
  }

  function clearSelection() {
    markedFiles.set(new Set());
  }

  async function handleTrash() {
    const paths = Array.from($markedFiles);
    if (paths.length === 0) return;
    showConfirm = false;
    try {
      const result = await trashFiles(paths);
      showToast(`Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`);
      markedFiles.set(new Set());
      const updated = await getResults();
      if (updated) report.set(updated);
    } catch (e) {
      showToast(`${e}`, 'error');
    }
  }

  async function handleUndo() {
    try {
      const result = await undoLast();
      showToast(`Restored ${result.restored} files`);
    } catch (e) {
      showToast(`Undo failed: ${e}`, 'error');
    }
  }

  async function handleExportJson() {
    const data = await exportJson();
    downloadFile('duplff-report.json', data, 'application/json');
  }

  async function handleExportCsv() {
    const data = await exportCsv();
    downloadFile('duplff-report.csv', data, 'text/csv');
  }

  function downloadFile(name: string, content: string, type: string) {
    const blob = new Blob([content], { type });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = name;
    a.click();
    URL.revokeObjectURL(url);
  }

  function handleSort(mode: SortMode) {
    sortMode.set(mode);
  }

  function newScan() {
    report.set(null);
    markedFiles.set(new Set());
    currentScreen.set('setup');
  }
</script>

<div class="flex flex-col h-screen">
  <!-- Header -->
  <div class="flex items-center justify-between px-4 py-2.5 border-b border-gray-800">
    <div class="flex items-center gap-4 text-xs text-gray-500">
      <span><span class="font-mono text-gray-300">{groups.length}</span> groups</span>
      <span><span class="font-mono text-gray-300">{totalDuplicates}</span> duplicates</span>
      <span class="text-delete"><span class="font-mono">{formatBytes(totalWasted)}</span> wasted</span>
    </div>
    <button on:click={newScan} class="text-xs text-gray-600 hover:text-gray-400 transition-colors">New Scan</button>
  </div>

  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-800/50">
    <input type="text" placeholder="Filter..." bind:value={$filterText}
      class="bg-transparent border border-gray-800 rounded-md px-2.5 py-1 text-xs flex-1 focus:border-gray-600 focus:outline-none placeholder-gray-700" />
    <select value={$sortMode} on:change={(e) => handleSort(e.currentTarget.value)}
      class="bg-transparent border border-gray-800 rounded-md px-2 py-1 text-xs text-gray-400">
      <option value="wasted">Wasted</option>
      <option value="size">Size</option>
      <option value="files">Files</option>
      <option value="path">Path</option>
    </select>
  </div>

  <!-- Table -->
  <div class="flex-1 overflow-auto">
    <GroupTable {groups} sortMode={$sortMode} filterText={$filterText} onSelectGroup={selectGroup} />
  </div>

  <!-- Action Bar -->
  <div class="flex items-center justify-between px-4 py-2 border-t border-gray-800">
    <div class="flex gap-1.5">
      <button on:click={autoSelectAll} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Select All</button>
      <button on:click={clearSelection} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Clear</button>
      {#if $markedFiles.size > 0}
        <button on:click={() => showConfirm = true} class="text-xs bg-delete/10 text-delete hover:bg-delete/20 px-2.5 py-1 rounded transition-colors">
          Trash {$markedFiles.size}
        </button>
      {/if}
    </div>
    <div class="flex gap-1.5">
      <button on:click={handleUndo} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Undo</button>
      <button on:click={handleExportJson} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">JSON</button>
      <button on:click={handleExportCsv} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">CSV</button>
    </div>
  </div>

  <!-- Toast -->
  {#if toast}
    <div class="fixed bottom-4 left-1/2 -translate-x-1/2 text-xs px-3 py-1.5 rounded-md shadow-lg transition-opacity
      {toast.type === 'error' ? 'bg-delete/20 text-delete border border-delete/30' : 'bg-gray-800 text-gray-300 border border-gray-700'}">
      {toast.message}
    </div>
  {/if}
</div>

{#if showConfirm}
  <ConfirmDialog
    title="Trash files?"
    message="Move {$markedFiles.size} files to the OS trash? This can be undone."
    onConfirm={handleTrash}
    onCancel={() => showConfirm = false}
  />
{/if}
