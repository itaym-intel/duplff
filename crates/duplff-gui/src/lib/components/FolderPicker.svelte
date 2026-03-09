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
  <span class="block text-[11px] font-medium text-text-muted uppercase tracking-widest mb-2">{label}</span>
  {#if folders.length > 0}
    <div class="flex flex-wrap gap-2 mb-2.5">
      {#each folders as folder, i}
        <span class="inline-flex items-center gap-2 bg-surface border border-border rounded-lg px-3 py-2 text-sm group
          hover:border-border transition-colors" title={folder}>
          <span class="w-1.5 h-1.5 rounded-full bg-active/60 shrink-0"></span>
          <span class="font-mono text-text-secondary max-w-52 truncate">{basename(folder)}</span>
          <button
            class="text-text-muted hover:text-delete transition-colors -mr-0.5 text-base leading-none"
            onclick={() => removeFolder(i)}
            aria-label="Remove {folder}"
          >&times;</button>
        </span>
      {/each}
    </div>
  {/if}
  <button
    class="w-full border border-dashed border-border rounded-xl py-5 text-sm text-text-muted
      hover:border-active/30 hover:text-text-secondary hover:bg-active/3 transition-all"
    onclick={addFolder}
  >
    + Add folder
  </button>
</div>
