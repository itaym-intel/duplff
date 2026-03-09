<script lang="ts">
  import { readFilePreview } from '$lib/api';
  import { fileExtension } from '$lib/format';
  import hljs from 'highlight.js/lib/core';

  import javascript from 'highlight.js/lib/languages/javascript';
  import typescript from 'highlight.js/lib/languages/typescript';
  import python from 'highlight.js/lib/languages/python';
  import rust from 'highlight.js/lib/languages/rust';
  import css from 'highlight.js/lib/languages/css';
  import xml from 'highlight.js/lib/languages/xml';
  import json from 'highlight.js/lib/languages/json';
  import bash from 'highlight.js/lib/languages/bash';
  import yaml from 'highlight.js/lib/languages/yaml';
  import sql from 'highlight.js/lib/languages/sql';
  import cpp from 'highlight.js/lib/languages/cpp';
  import go from 'highlight.js/lib/languages/go';
  import java from 'highlight.js/lib/languages/java';
  import markdown from 'highlight.js/lib/languages/markdown';

  hljs.registerLanguage('javascript', javascript);
  hljs.registerLanguage('typescript', typescript);
  hljs.registerLanguage('python', python);
  hljs.registerLanguage('rust', rust);
  hljs.registerLanguage('css', css);
  hljs.registerLanguage('xml', xml);
  hljs.registerLanguage('json', json);
  hljs.registerLanguage('bash', bash);
  hljs.registerLanguage('yaml', yaml);
  hljs.registerLanguage('sql', sql);
  hljs.registerLanguage('cpp', cpp);
  hljs.registerLanguage('go', go);
  hljs.registerLanguage('java', java);
  hljs.registerLanguage('markdown', markdown);

  const extToLang: Record<string, string> = {
    js: 'javascript', jsx: 'javascript', mjs: 'javascript',
    ts: 'typescript', tsx: 'typescript',
    py: 'python',
    rs: 'rust',
    css: 'css', scss: 'css',
    html: 'xml', svelte: 'xml', vue: 'xml', xml: 'xml',
    json: 'json',
    sh: 'bash', bash: 'bash', zsh: 'bash',
    yaml: 'yaml', yml: 'yaml', toml: 'yaml',
    sql: 'sql',
    c: 'cpp', cpp: 'cpp', h: 'cpp', hpp: 'cpp',
    go: 'go',
    java: 'java',
    md: 'markdown',
  };

  let { path }: { path: string } = $props();

  let content = $state<string | null>(null);
  let error = $state<string | null>(null);
  let highlighted = $state('');

  $effect(() => {
    content = null;
    error = null;
    readFilePreview(path)
      .then((text) => {
        content = text;
        const ext = fileExtension(path);
        const lang = extToLang[ext];
        if (lang && hljs.getLanguage(lang)) {
          highlighted = hljs.highlight(text, { language: lang }).value;
        } else {
          highlighted = '';
        }
      })
      .catch((e) => { error = String(e); });
  });
</script>

<div class="mt-2 rounded-xl border border-border-subtle overflow-hidden animate-[fadeIn_150ms_ease-out]"
  style="background: oklch(0.13 0.005 264.5);">
  {#if error}
    <p class="text-text-muted text-xs p-4">{error}</p>
  {:else if content === null}
    <div class="flex items-center gap-2 p-4">
      <div class="w-3 h-3 rounded-full border-2 border-active/30 border-t-active animate-spin"></div>
      <span class="text-text-muted text-xs">Loading preview...</span>
    </div>
  {:else if highlighted}
    <pre class="p-4 text-xs font-mono leading-relaxed overflow-x-auto max-h-80"><code>{@html highlighted}</code></pre>
  {:else}
    <pre class="p-4 text-xs font-mono text-text-secondary leading-relaxed overflow-x-auto max-h-80">{content}</pre>
  {/if}
</div>

<style>
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>
