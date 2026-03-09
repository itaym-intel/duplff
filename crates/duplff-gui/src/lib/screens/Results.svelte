<script lang="ts">
  import { report, markedFiles, filterText, sortMode, ignoredGroups } from '$lib/stores';
  import { trashFiles, exportJson, exportCsv, undoLast, getResults } from '$lib/api';
  import { formatBytes } from '$lib/format';
  import StatsBar from '$lib/components/StatsBar.svelte';
  import VirtualGroupList from '$lib/components/VirtualGroupList.svelte';
  import SelectionBar from '$lib/components/SelectionBar.svelte';
  import KeyboardHandler from '$lib/components/KeyboardHandler.svelte';
  import ShortcutHelp from '$lib/components/ShortcutHelp.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import Toast from '$lib/components/Toast.svelte';

  let showConfirm = $state(false);
  let showHelp = $state(false);
  let toast = $state<{ message: string; type: 'info' | 'error' } | null>(null);
  let toastTimer: ReturnType<typeof setTimeout>;

  let groups = $derived($report?.groups ?? []);

  let filteredGroups = $derived.by(() => {
    const text = $filterText.toLowerCase();
    const ignored = $ignoredGroups;
    return groups
      .map((g, i) => ({ group: g, index: i }))
      .filter(({ group, index }) => {
        if (ignored.has(index)) return false;
        if (!text) return true;
        return group.keep.entry.path.toLowerCase().includes(text) ||
          group.duplicates.some(d => d.entry.path.toLowerCase().includes(text));
      });
  });

  let sortedGroups = $derived.by(() => {
    return [...filteredGroups].sort((a, b) => {
      switch ($sortMode) {
        case 'wasted':
          return (b.group.size * b.group.duplicates.length) - (a.group.size * a.group.duplicates.length);
        case 'size':
          return b.group.size - a.group.size;
        case 'files':
          return (b.group.duplicates.length + 1) - (a.group.duplicates.length + 1);
        case 'path':
          return a.group.keep.entry.path.localeCompare(b.group.keep.entry.path);
        default:
          return 0;
      }
    });
  });

  function showToast(message: string, type: 'info' | 'error' = 'info') {
    clearTimeout(toastTimer);
    toast = { message, type };
    toastTimer = setTimeout(() => { toast = null; }, 3000);
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
      const updated = await getResults();
      if (updated) report.set(updated);
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

  function requestTrash() {
    if ($markedFiles.size > 0) showConfirm = true;
  }

  function focusFilter() {
    const input = document.querySelector<HTMLInputElement>('input[placeholder="Filter groups..."]');
    input?.focus();
  }
</script>

<div class="flex flex-col h-screen">
  <StatsBar onExportJson={handleExportJson} onExportCsv={handleExportCsv} />

  {#if sortedGroups.length === 0}
    <div class="flex-1 flex items-center justify-center">
      <p class="text-text-muted text-sm">
        {$filterText ? 'No groups match your filter.' : 'No duplicates found.'}
      </p>
    </div>
  {:else}
    <VirtualGroupList groups={sortedGroups} />
  {/if}

  <SelectionBar onTrash={requestTrash} onUndo={handleUndo} />

  <KeyboardHandler
    groups={sortedGroups}
    onTrash={requestTrash}
    onShowHelp={() => showHelp = true}
    onFocusFilter={focusFilter}
  />
</div>

{#if toast}
  <Toast message={toast.message} type={toast.type} />
{/if}

{#if showConfirm}
  <ConfirmDialog
    title="Trash files?"
    message="Move {$markedFiles.size} files to the OS trash. This can be undone."
    confirmLabel="Trash {$markedFiles.size}"
    onConfirm={handleTrash}
    onCancel={() => showConfirm = false}
  />
{/if}

{#if showHelp}
  <ShortcutHelp onClose={() => showHelp = false} />
{/if}
