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
<div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" onclick={onCancel}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="bg-gray-900 border border-border rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl" onclick={(e) => e.stopPropagation()}>
    <h3 class="text-base font-semibold mb-1.5">{title}</h3>
    <p class="text-text-secondary text-sm mb-6 leading-relaxed">{message}</p>
    <div class="flex gap-3 justify-end">
      <button onclick={onCancel}
        class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary rounded-lg transition-colors">
        Cancel
      </button>
      <button onclick={onConfirm}
        class="px-4 py-2 text-sm bg-delete hover:bg-red-600 text-white rounded-lg transition-colors font-medium">
        {confirmLabel}
      </button>
    </div>
  </div>
</div>
