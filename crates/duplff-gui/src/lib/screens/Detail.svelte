<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles } from '$lib/stores';
  import { trashFiles, undoLast, getResults } from '$lib/api';
  import { formatBytes } from '$lib/format';
  import FileList from '$lib/components/FileList.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

  let showConfirm = false;
  let toast: { message: string; type: 'info' | 'error' } | null = null;
  let toastTimer: ReturnType<typeof setTimeout>;

  $: group = $report?.groups[$selectedGroup] ?? null;
  $: groupMarked = group
    ? new Set([...$markedFiles].filter(p => group!.duplicates.some(d => d.entry.path === p)))
    : new Set<string>();

  function showToast(message: string, type: 'info' | 'error' = 'info') {
    clearTimeout(toastTimer);
    toast = { message, type };
    toastTimer = setTimeout(() => { toast = null; }, 3000);
  }

  function toggleFile(path: string) {
    markedFiles.update(set => {
      const next = new Set(set);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  function selectAllDuplicates() {
    if (!group) return;
    markedFiles.update(set => {
      const next = new Set(set);
      for (const dup of group!.duplicates) next.add(dup.entry.path);
      return next;
    });
  }

  function deselectAll() {
    if (!group) return;
    markedFiles.update(set => {
      const next = new Set(set);
      for (const dup of group!.duplicates) next.delete(dup.entry.path);
      return next;
    });
  }

  async function handleTrash() {
    showConfirm = false;
    const paths = Array.from(groupMarked);
    try {
      const result = await trashFiles(paths);
      showToast(`Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`);
      markedFiles.update(set => {
        const next = new Set(set);
        for (const p of paths) next.delete(p);
        return next;
      });
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
</script>

<div class="flex flex-col h-screen">
  <!-- Header -->
  <div class="flex items-center gap-3 px-4 py-2.5 border-b border-gray-800">
    <button on:click={() => currentScreen.set('results')} class="text-xs text-gray-600 hover:text-gray-400 transition-colors">&larr; Results</button>
    {#if group}
      <span class="text-gray-800">&#47;</span>
      <div class="flex items-center gap-3 text-xs text-gray-500">
        <span>Size <span class="font-mono text-gray-400">{formatBytes(group.size)}</span></span>
        <span>Files <span class="font-mono text-gray-400">{group.duplicates.length + 1}</span></span>
        <span class="text-delete">Wasted <span class="font-mono">{formatBytes(group.size * group.duplicates.length)}</span></span>
      </div>
      <span class="ml-auto text-[10px] font-mono text-gray-700 select-all" title="Full hash">{Array.from(group.hash.slice(0, 8), b => b.toString(16).padStart(2, '0')).join('')}...</span>
    {/if}
  </div>

  <!-- File List -->
  <div class="flex-1 overflow-auto p-4">
    {#if group}
      <FileList keep={group.keep} duplicates={group.duplicates} markedPaths={groupMarked} onToggle={toggleFile} />
    {:else}
      <p class="text-gray-600 text-sm">Group not found.</p>
    {/if}
  </div>

  <!-- Action Bar -->
  <div class="flex items-center justify-between px-4 py-2 border-t border-gray-800">
    <div class="flex gap-1.5">
      <button on:click={selectAllDuplicates} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Select All</button>
      <button on:click={deselectAll} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Clear</button>
    </div>
    <div class="flex gap-1.5">
      <button on:click={handleUndo} class="text-xs text-gray-500 hover:text-gray-300 px-2 py-1 rounded transition-colors">Undo</button>
      {#if groupMarked.size > 0}
        <button on:click={() => showConfirm = true} class="text-xs bg-delete/10 text-delete hover:bg-delete/20 px-2.5 py-1 rounded transition-colors">
          Trash {groupMarked.size}
        </button>
      {/if}
    </div>
  </div>

  <!-- Toast -->
  {#if toast}
    <div class="fixed bottom-4 left-1/2 -translate-x-1/2 text-xs px-3 py-1.5 rounded-md shadow-lg
      {toast.type === 'error' ? 'bg-delete/20 text-delete border border-delete/30' : 'bg-gray-800 text-gray-300 border border-gray-700'}">
      {toast.message}
    </div>
  {/if}
</div>

{#if showConfirm}
  <ConfirmDialog
    title="Trash selected files?"
    message="Move {groupMarked.size} files to the OS trash? This can be undone."
    onConfirm={handleTrash}
    onCancel={() => showConfirm = false}
  />
{/if}
