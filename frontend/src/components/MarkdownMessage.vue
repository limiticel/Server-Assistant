<script setup lang="ts">
import hljs from 'highlight.js/lib/common'
import MarkdownIt from 'markdown-it'
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'

const props = defineProps<{
  content: string
}>()

const root = ref<HTMLElement | null>(null)

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

const markdown = new MarkdownIt({
  breaks: true,
  html: false,
  linkify: true
})

const defaultLinkOpen =
  markdown.renderer.rules.link_open ??
  ((tokens, idx, options, _env, self) => self.renderToken(tokens, idx, options))

markdown.renderer.rules.link_open = (tokens, idx, options, env, self) => {
  const token = tokens[idx]
  const targetIndex = token.attrIndex('target')
  const relIndex = token.attrIndex('rel')

  if (targetIndex < 0) {
    token.attrPush(['target', '_blank'])
  } else {
    token.attrs![targetIndex][1] = '_blank'
  }

  if (relIndex < 0) {
    token.attrPush(['rel', 'noopener noreferrer'])
  } else {
    token.attrs![relIndex][1] = 'noopener noreferrer'
  }

  return defaultLinkOpen(tokens, idx, options, env, self)
}

markdown.renderer.rules.fence = (tokens, idx) => {
  const token = tokens[idx]
  const code = token.content
  const language = token.info.trim().split(/\s+/)[0]
  const label = language || 'codigo'
  const highlighted =
    language && hljs.getLanguage(language)
      ? hljs.highlight(code, { language, ignoreIllegals: true }).value
      : escapeHtml(code)

  return `
    <div class="markdown-code-block">
      <div class="markdown-code-header">
        <span>${escapeHtml(label)}</span>
        <button class="markdown-copy-button" type="button">Copiar</button>
      </div>
      <pre><code class="hljs language-${escapeHtml(language)}">${highlighted}</code></pre>
    </div>
  `
}

function handleCopyClick(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof HTMLElement)) return

  const button = target.closest<HTMLButtonElement>('.markdown-copy-button')
  if (!button) return

  const block = button.closest('.markdown-code-block')
  const code = block?.querySelector('code')?.textContent ?? ''
  if (!code) return

  void navigator.clipboard.writeText(code).then(() => {
    const previousLabel = button.textContent ?? 'Copiar'
    button.textContent = 'Copiado'
    window.setTimeout(() => {
      button.textContent = previousLabel
    }, 1400)
  })
}

onMounted(() => {
  root.value?.addEventListener('click', handleCopyClick)
})

onBeforeUnmount(() => {
  root.value?.removeEventListener('click', handleCopyClick)
})

const rendered = computed(() => markdown.render(props.content || ''))
</script>

<template>
  <div ref="root" class="markdown-message" v-html="rendered" />
</template>
