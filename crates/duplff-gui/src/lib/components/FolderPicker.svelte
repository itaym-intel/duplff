<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  let { folders = $bindable([]), label = 'Folders' }: { folders: string[]; label?: string } = $props();

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
  <span class="block text-xs font-medium text-text-muted uppercase tracking-wider mb-2">{label}</span>
  {#if folders.length > 0}
    <div class="flex flex-wrap gap-1.5 mb-2">
      {#each folders as folder, i}
        <span class="inline-flex items-center gap-1.5 bg-surface border border-border rounded-lg px-3 py-1.5 text-sm group" title={folder}>
          <span class="font-mono text-text-secondary max-w-48 truncate">{basename(folder)}</span>
          <button
            class="text-text-muted hover:text-delete transition-colors -mr-0.5"
            onclick={() => removeFolder(i)}
            aria-label="Remove {folder}"
          >&times;</button>
        </span>
      {/each}
    </div>
  {/if}
  <button
    class="w-full border border-dashed border-border rounded-xl py-5 text-sm text-text-muted
      hover:border-text-muted hover:text-text-secondary transition-colors"
    onclick={addFolder}
  >
    + Add folder
  </button>
</div>
