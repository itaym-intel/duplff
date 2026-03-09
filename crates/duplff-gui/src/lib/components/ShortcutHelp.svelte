<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();

  const shortcuts = [
    { keys: ['j', 'k'], description: 'Navigate groups' },
    { keys: ['↓', '↑'], description: 'Navigate groups' },
    { keys: ['Enter', '→'], description: 'Expand / collapse group' },
    { keys: ['Esc', '←'], description: 'Collapse group' },
    { keys: ['Space'], description: 'Toggle all duplicates' },
    { keys: ['d', 'Del'], description: 'Trash selected files' },
    { keys: ['/'], description: 'Focus filter' },
    { keys: ['?'], description: 'Show this help' },
  ];
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape' || e.key === '?') { e.preventDefault(); onClose(); }}} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50
  animate-[fadeIn_100ms_ease-out]" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="bg-surface-raised border border-border rounded-2xl p-6 max-w-sm w-full mx-4
    shadow-2xl shadow-black/40 animate-[scaleIn_150ms_ease-out]" onclick={(e) => e.stopPropagation()}>
    <div class="flex items-center gap-2.5 mb-5">
      <span class="text-[11px] font-medium text-text-muted uppercase tracking-widest">Keyboard shortcuts</span>
    </div>
    <div class="space-y-2.5">
      {#each shortcuts as { keys, description }}
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-secondary">{description}</span>
          <div class="flex gap-1.5">
            {#each keys as key}
              <kbd class="px-2 py-1 bg-surface border border-border-subtle rounded-md text-[11px] font-mono
                text-text-primary min-w-[26px] text-center">{key}</kbd>
            {/each}
          </div>
        </div>
      {/each}
    </div>
    <div class="mt-6 pt-4 border-t border-border-subtle text-right">
      <button onclick={onClose}
        class="text-xs text-text-muted hover:text-text-secondary transition-colors uppercase tracking-widest">
        Close
      </button>
    </div>
  </div>
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes scaleIn {
    from { opacity: 0; transform: scale(0.96); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
