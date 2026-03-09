<script lang="ts">
  let { message, type = 'info' }: { message: string; type?: 'info' | 'error' } = $props();
</script>

<div
  class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 text-sm px-5 py-3 rounded-xl shadow-2xl shadow-black/30
    backdrop-blur-md animate-[fadeUp_200ms_ease-out]
    {type === 'error'
      ? 'bg-delete/12 text-delete border border-delete/20'
      : 'bg-surface-raised border border-border text-text-primary'}"
  role="status"
>
  <span class="font-medium">{message}</span>
</div>

<style>
  @keyframes fadeUp {
    from { opacity: 0; transform: translate(-50%, 8px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
</style>
