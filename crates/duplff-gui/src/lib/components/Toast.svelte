<script lang="ts">
  let { message, type = 'info' }: { message: string; type?: 'info' | 'error' } = $props();
</script>

<div
  class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 text-sm px-4 py-2 rounded-lg shadow-xl backdrop-blur-sm
    animate-[fadeUp_200ms_ease-out]
    {type === 'error'
      ? 'bg-delete/15 text-delete border border-delete/20'
      : 'bg-surface border border-border text-text-primary'}"
  role="status"
>
  {message}
</div>

<style>
  @keyframes fadeUp {
    from { opacity: 0; transform: translate(-50%, 8px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
</style>
