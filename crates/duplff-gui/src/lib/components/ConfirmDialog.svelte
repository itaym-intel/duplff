<script lang="ts">
  let { title, message, confirmLabel = 'Confirm', onConfirm, onCancel }: {
    title: string;
    message: string;
    confirmLabel?: string;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') onCancel();
    if (e.key === 'Enter') onConfirm();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50
  animate-[fadeIn_100ms_ease-out]" onclick={onCancel}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="bg-surface-raised border border-border rounded-2xl p-6 max-w-md w-full mx-4
    shadow-2xl shadow-black/40 animate-[scaleIn_150ms_ease-out]" onclick={(e) => e.stopPropagation()}>
    <h3 class="text-base font-semibold mb-2">{title}</h3>
    <p class="text-text-secondary text-sm mb-8 leading-relaxed">{message}</p>
    <div class="flex gap-3 justify-end">
      <button onclick={onCancel}
        class="px-4 py-2.5 text-sm text-text-secondary hover:text-text-primary rounded-xl
          hover:bg-surface-hover transition-colors">
        Cancel
      </button>
      <button onclick={onConfirm}
        class="px-5 py-2.5 text-sm bg-delete/90 hover:bg-delete text-white rounded-xl transition-all
          font-medium shadow-lg shadow-delete/15 hover:shadow-delete/25 active:scale-[0.98]">
        {confirmLabel}
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
