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
</script>

<div class="space-y-2">
  <label class="text-sm font-medium text-gray-300">{label}</label>
  <div class="space-y-1">
    {#each folders as folder, i}
      <div class="flex items-center gap-2 bg-gray-800 rounded px-3 py-1.5 text-sm">
        <span class="truncate flex-1 font-mono">{folder}</span>
        <button
          class="text-gray-500 hover:text-delete shrink-0"
          on:click={() => removeFolder(i)}
        >×</button>
      </div>
    {/each}
  </div>
  <button
    class="w-full border-2 border-dashed border-gray-600 rounded-lg py-3 text-gray-400 hover:border-active hover:text-active transition-colors"
    on:click={addFolder}
  >
    + Add folder
  </button>
</div>
