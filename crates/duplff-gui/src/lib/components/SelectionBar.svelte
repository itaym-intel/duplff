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
  <div class="flex items-center justify-between px-5 py-3 border-t border-border bg-surface/80 backdrop-blur-md
    animate-[slideUp_150ms_ease-out]">
    <div class="flex items-center gap-4">
      <span class="text-sm text-text-muted">
        <span class="font-mono font-medium text-text-primary">{count}</span>
        {count === 1 ? 'file' : 'files'} selected
      </span>
      <button onclick={clear}
        class="text-[11px] text-text-muted hover:text-text-secondary transition-colors uppercase tracking-widest">
        Clear
      </button>
    </div>
    <div class="flex items-center gap-2.5">
      <button onclick={onUndo}
        class="text-sm text-text-muted hover:text-text-secondary px-3.5 py-2 rounded-lg
          hover:bg-surface-hover transition-colors">
        Undo
      </button>
      <button onclick={onTrash}
        class="text-sm bg-delete/90 hover:bg-delete text-white font-medium px-5 py-2 rounded-lg
          transition-all shadow-lg shadow-delete/15 hover:shadow-delete/25 active:scale-[0.98]">
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
