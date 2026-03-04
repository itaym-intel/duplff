<script lang="ts">
  export let title: string;
  export let message: string;
  export let onConfirm: () => void;
  export let onCancel: () => void;
</script>

<div class="fixed inset-0 bg-black/60 flex items-center justify-center z-50">
  <div class="bg-gray-800 border border-gray-700 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
    <h3 class="text-lg font-bold mb-2">{title}</h3>
    <p class="text-gray-400 text-sm mb-6">{message}</p>
    <div class="flex gap-3 justify-end">
      <button on:click={onCancel} class="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm">Cancel</button>
      <button on:click={onConfirm} class="px-4 py-2 bg-delete hover:bg-red-600 rounded text-sm">Trash</button>
    </div>
  </div>
</div>
