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
    <div class="flex flex-col gap-1.5 mb-2.5">
      {#each folders as folder, i}
        <div class="flex items-center gap-3 border border-border-subtle rounded-lg px-3 py-2.5 group
          hover:border-border transition-colors" title={folder}>
          <span class="text-text-muted text-xs font-mono shrink-0">/</span>
          <span class="font-mono text-sm text-text-secondary truncate flex-1">{basename(folder)}</span>
          <span class="font-mono text-[11px] text-text-muted/60 truncate max-w-40 hidden group-hover:block">{folder}</span>
          <button
            class="text-text-muted/50 hover:text-delete transition-colors text-base leading-none shrink-0"
            onclick={() => removeFolder(i)}
            aria-label="Remove {folder}"
          >&times;</button>
        </div>
      {/each}
    </div>
  {/if}
  <button
    class="w-full border border-dashed border-border rounded-xl py-5 text-sm text-text-muted
      hover:border-keep/40 hover:bg-keep/5 transition-all"
    onclick={addFolder}
  >
    + Add folder
  </button>
</div>
