<script lang="ts">
  import { report, currentScreen, selectedGroup, markedFiles } from '$lib/stores';
  import { trashFiles, undoLast, getResults } from '$lib/api';
  import { formatBytes } from '$lib/format';
  import FileList from '$lib/components/FileList.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

  let showConfirm = false;
  let statusMessage: string | null = null;

  $: group = $report?.groups[$selectedGroup] ?? null;
  $: groupMarked = group
    ? new Set([...$markedFiles].filter(p => group!.duplicates.some(d => d.entry.path === p)))
    : new Set<string>();

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
      statusMessage = `Trashed ${result.count} files (${formatBytes(result.bytes_reclaimed)})`;
      markedFiles.update(set => {
        const next = new Set(set);
        for (const p of paths) next.delete(p);
        return next;
      });
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
</script>

<div class="flex flex-col h-screen">
  <div class="bg-gray-800 border-b border-gray-700 px-6 py-3">
    <div class="flex items-center gap-4">
      <button on:click={() => currentScreen.set('results')} class="text-gray-400 hover:text-white text-sm">← Back</button>
      {#if group}
        <div class="flex items-center gap-4 text-sm text-gray-400">
          <span>Size: <span class="text-gray-200 font-mono">{formatBytes(group.size)}</span></span>
          <span>Files: <span class="text-gray-200 font-mono">{group.duplicates.length + 1}</span></span>
          <span class="text-delete">Wasted: <span class="font-mono">{formatBytes(group.size * group.duplicates.length)}</span></span>
        </div>
      {/if}
    </div>
  </div>

  <div class="flex-1 overflow-auto p-6">
    {#if group}
      <FileList keep={group.keep} duplicates={group.duplicates} markedPaths={groupMarked} onToggle={toggleFile} />
    {:else}
      <p class="text-gray-500">Group not found.</p>
    {/if}
  </div>

  <div class="bg-gray-800 border-t border-gray-700 px-6 py-3 flex items-center justify-between">
    <div class="flex gap-2">
      <button on:click={selectAllDuplicates} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Select All</button>
      <button on:click={deselectAll} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Deselect All</button>
    </div>
    <div class="flex gap-2">
      <button on:click={handleUndo} class="text-sm bg-gray-700 hover:bg-gray-600 px-3 py-1.5 rounded">Undo</button>
      {#if groupMarked.size > 0}
        <button on:click={() => showConfirm = true} class="text-sm bg-delete hover:bg-red-600 px-3 py-1.5 rounded">
          Trash {groupMarked.size} files
        </button>
      {/if}
    </div>
  </div>

  {#if statusMessage}
    <div class="bg-gray-800 border-t border-gray-700 px-6 py-2 text-sm text-gray-400">{statusMessage}</div>
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
