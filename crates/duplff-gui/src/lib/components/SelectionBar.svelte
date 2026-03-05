<script lang="ts">
  import { markedFiles } from '$lib/stores';

  let {
    onTrash,
    onUndo,
  }: {
    onTrash: () => void;
    onUndo: () => void;
  } = $props();

  let count = $derived($markedFiles.size);

  function clear() {
    markedFiles.set(new Set());
  }
</script>

{#if count > 0}
  <div class="flex items-center justify-between px-4 py-2.5 border-t border-border bg-gray-950/80 backdrop-blur-sm
    animate-[slideUp_150ms_ease-out]">
    <div class="flex items-center gap-3">
      <span class="text-sm text-text-secondary">
        <span class="font-mono font-medium text-text-primary">{count}</span>
        {count === 1 ? 'file' : 'files'} selected
      </span>
      <button onclick={clear} class="text-sm text-text-muted hover:text-text-secondary transition-colors">
        Clear
      </button>
    </div>
    <div class="flex items-center gap-2">
      <button onclick={onUndo} class="text-sm text-text-muted hover:text-text-secondary px-3 py-1.5 rounded-lg transition-colors">
        Undo
      </button>
      <button onclick={onTrash}
        class="text-sm bg-delete hover:bg-red-600 text-white font-medium px-4 py-1.5 rounded-lg transition-colors">
        Trash {count}
      </button>
    </div>
  </div>
{/if}

<style>
  @keyframes slideUp {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
