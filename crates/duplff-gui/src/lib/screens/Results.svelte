<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles, filterText, sortMode } from '$lib/stores';
  import { trashFiles, exportJson, exportCsv, undoLast, getResults } from '$lib/api';
  import { formatBytes } from '$lib/format';
  import GroupTable from '$lib/components/GroupTable.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import type { SortMode } from '$lib/types';

  let showConfirm = false;
  let statusMessage: string | null = null;

  $: groups = $report?.groups ?? [];
  $: totalDuplicates = $report?.total_duplicates ?? 0;
  $: totalWasted = $report?.total_wasted_bytes ?? 0;

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
      statusMessage = `Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`;
      markedFiles.set(new Set());
      const updated = await getResults();
      if (updated) report.set(updated);
    } catch (e) {
      statusMessage = `Error: ${e}`;
    }
  }

  async function handleUndo() {
    try {
      const result = await undoLast();
      statusMessage = `Restored ${result.restored} files`;
    } catch (e) {
      statusMessage = `Undo failed: ${e}`;
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
  <div class="bg-gray-800 border-b border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex items-center gap-6 text-sm">
      <span>{groups.length} groups</span>
      <span>{totalDuplicates} duplicates</span>
      <span class="text-delete">{formatBytes(totalWasted)} wasted</span>
    </div>
    <button on:click={newScan} class="text-sm text-gray-400 hover:text-white px-3 py-1">New Scan</button>
  </div>

  <div class="px-6 py-3 flex items-center gap-3 border-b border-gray-800">
    <input type="text" placeholder="Filter by path..." bind:value={$filterText}
      class="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm flex-1 focus:border-active focus:outline-none" />
    <select value={$sortMode} on:change={(e) => handleSort(e.currentTarget.value)}
      class="bg-gray-800 border border-gray-700 rounded px-3 py-1.5 text-sm">
      <option value="wasted">Sort: Wasted</option>
      <option value="size">Sort: Size</option>
      <option value="files">Sort: Files</option>
      <option value="path">Sort: Path</option>
    </select>
  </div>

  <div class="flex-1 overflow-auto">
    <GroupTable {groups} sortMode={$sortMode} filterText={$filterText} onSelectGroup={selectGroup} />
  </div>

  <div class="bg-gray-800 border-t border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex gap-2">
      <button on:click={autoSelectAll} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Select All Duplicates</button>
      <button on:click={clearSelection} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Clear Selection</button>
      {#if $markedFiles.size > 0}
        <button on:click={() => showConfirm = true} class="text-sm bg-delete hover:bg-red-600 px-3 py-1.5 rounded">
          Trash {$markedFiles.size} files
        </button>
      {/if}
    </div>
    <div class="flex gap-2">
      <button on:click={handleUndo} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Undo</button>
      <button on:click={handleExportJson} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">JSON</button>
      <button on:click={handleExportCsv} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">CSV</button>
    </div>
  </div>

  {#if statusMessage}
    <div class="bg-gray-800 border-t border-gray-700 px-6 py-2 text-sm text-gray-400">{statusMessage}</div>
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
