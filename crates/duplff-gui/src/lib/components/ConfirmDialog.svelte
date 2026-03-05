<script lang="ts">
  export let title: string;
  export let message: string;
  export let onConfirm: () => void;
  export let onCancel: () => void;
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50" on:click|self={onCancel}>
  <div class="bg-gray-800/95 border border-gray-700 rounded-lg p-5 max-w-sm w-full mx-4 shadow-2xl">
    <h3 class="text-sm font-semibold mb-1">{title}</h3>
    <p class="text-gray-500 text-xs mb-5">{message}</p>
    <div class="flex gap-2 justify-end">
      <button on:click={onCancel} class="px-3 py-1.5 text-xs text-gray-400 hover:text-gray-200 transition-colors">Cancel</button>
      <button on:click={onConfirm} class="px-3 py-1.5 text-xs bg-delete hover:bg-red-600 rounded transition-colors">Trash</button>
    </div>
  </div>
</div>
