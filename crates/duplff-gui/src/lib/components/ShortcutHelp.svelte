<script lang="ts">
  let { onClose }: { onClose: () => void } = $props();

  const shortcuts = [
    { keys: ['j', 'k'], description: 'Navigate groups' },
    { keys: ['Arrow Down', 'Up'], description: 'Navigate groups' },
    { keys: ['Enter', 'Right'], description: 'Expand / collapse group' },
    { keys: ['Escape', 'Left'], description: 'Collapse group' },
    { keys: ['Space'], description: 'Toggle all duplicates in group' },
    { keys: ['d', 'Delete'], description: 'Trash selected files' },
    { keys: ['/'], description: 'Focus filter' },
    { keys: ['?'], description: 'Show this help' },
  ];
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Escape' || e.key === '?') { e.preventDefault(); onClose(); }}} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onclick={onClose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="bg-gray-900 border border-border rounded-xl p-6 max-w-sm w-full mx-4 shadow-2xl" onclick={(e) => e.stopPropagation()}>
    <h3 class="text-base font-semibold mb-4">Keyboard Shortcuts</h3>
    <div class="space-y-2">
      {#each shortcuts as { keys, description }}
        <div class="flex items-center justify-between">
          <span class="text-sm text-text-secondary">{description}</span>
          <div class="flex gap-1">
            {#each keys as key}
              <kbd class="px-2 py-0.5 bg-surface border border-border rounded text-xs font-mono text-text-primary">{key}</kbd>
            {/each}
          </div>
        </div>
      {/each}
    </div>
    <div class="mt-5 text-right">
      <button onclick={onClose} class="text-sm text-text-muted hover:text-text-secondary transition-colors">Close</button>
    </div>
  </div>
</div>
