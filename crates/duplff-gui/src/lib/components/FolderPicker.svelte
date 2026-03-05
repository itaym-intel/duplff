<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  export let folders: string[] = [];
  export let label = 'Folders';

  async function addFolder() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: `Select ${label.toLowerCase()}`,
    });
    if (selected && !folders.includes(selected as string)) {
      folders = [...folders, selected as string];
    }
  }

  function removeFolder(index: number) {
    folders = folders.filter((_, i) => i !== index);
  }

  function basename(path: string): string {
    return path.split('/').pop() || path;
  }
</script>

<div>
  <span class="block text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">{label}</span>
  {#if folders.length > 0}
    <div class="flex flex-wrap gap-1.5 mb-2">
      {#each folders as folder, i}
        <span class="inline-flex items-center gap-1.5 bg-gray-800 border border-gray-700 rounded-md px-2.5 py-1 text-xs group" title={folder}>
          <span class="font-mono text-gray-300 max-w-48 truncate">{basename(folder)}</span>
          <button
            class="text-gray-600 hover:text-delete transition-colors -mr-0.5"
            on:click={() => removeFolder(i)}
            aria-label="Remove {folder}"
          >&times;</button>
        </span>
      {/each}
    </div>
  {/if}
  <button
    class="w-full border border-dashed border-gray-700 rounded-lg py-4 text-sm text-gray-500 hover:border-gray-500 hover:text-gray-400 transition-colors"
    on:click={addFolder}
  >
    + Add folder
  </button>
</div>
