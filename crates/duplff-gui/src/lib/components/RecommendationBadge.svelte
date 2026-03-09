<script lang="ts">
  import type { Confidence } from '$lib/types';
  import { confidenceColor } from '$lib/format';

  let { reason, confidence }: { reason: string; confidence: Confidence } = $props();

  const reasonDetails: Record<string, { label: string; icon: string }> = {
    PriorityPath: { label: 'Priority directory', icon: '~' },
    DeepestPath: { label: 'Deepest path', icon: '/' },
    NewestModification: { label: 'Newest file', icon: '+' },
    LexicographicFirst: { label: 'First alphabetically', icon: 'a' },
  };

  let detail = $derived(reasonDetails[reason] || { label: reason, icon: '?' });
  let color = $derived(confidenceColor(confidence));
</script>

<div class="flex flex-wrap items-center gap-2 text-[11px]">
  <span class="inline-flex items-center gap-1.5 bg-keep/8 text-keep/90 px-2.5 py-1 rounded-md">
    <span class="font-mono text-[10px] opacity-60">{detail.icon}</span>
    {detail.label}
  </span>
  <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-surface-raised {color}">
    <span class="opacity-60">Confidence:</span>
    <span class="font-medium capitalize">{confidence}</span>
  </span>
</div>
