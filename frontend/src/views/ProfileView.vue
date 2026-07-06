<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { http } from '../api/http'
import AppShell from '../layouts/AppShell.vue'
import { applyTheme, getSavedTheme, type ThemeMode } from '../theme'

const theme = ref<ThemeMode>(getSavedTheme())
const saving = ref(false)
const saved = ref(false)
const chatContext = ref({
  compaction_enabled: true,
  max_messages: 20,
  keep_last_messages: 8,
  max_summary_chars: 4000
})

onMounted(async () => {
  const { data } = await http.get('/api/settings/chat-context')
  chatContext.value = data
})

function saveProfile() {
  applyTheme(theme.value)
}

async function saveChatContext() {
  saving.value = true
  saved.value = false
  try {
    const { data } = await http.put('/api/settings/chat-context', {
      compaction_enabled: chatContext.value.compaction_enabled,
      max_messages: Number(chatContext.value.max_messages),
      keep_last_messages: Number(chatContext.value.keep_last_messages),
      max_summary_chars: Number(chatContext.value.max_summary_chars)
    })
    chatContext.value = data
    saved.value = true
  } finally {
    saving.value = false
  }
}
</script>

<template>
  <AppShell>
    <section class="space-y-6 p-6">
      <h1 class="mb-4 text-2xl font-semibold">Perfil</h1>
      <div class="max-w-xl space-y-3 rounded-lg bg-white p-5 shadow-sm dark:bg-slate-900">
        <input class="w-full rounded-md border border-gray-300 px-3 py-2" placeholder="Nome" />
        <select v-model="theme" class="w-full rounded-md border border-gray-300 px-3 py-2" @change="saveProfile">
          <option value="light">Tema claro</option>
          <option value="dark">Tema escuro</option>
        </select>
        <button class="rounded-md bg-brand px-4 py-2 font-medium text-white" @click="saveProfile">Salvar</button>
      </div>

      <div class="max-w-xl space-y-4 rounded-lg bg-white p-5 shadow-sm dark:bg-slate-900">
        <div>
          <h2 class="text-lg font-semibold">Contexto do chat</h2>
          <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">
            Compacta mensagens antigas antes de enviar para a IA, mantendo as mensagens recentes completas.
          </p>
        </div>

        <label class="flex items-center gap-3 rounded-md border border-gray-200 p-3 text-sm dark:border-slate-700">
          <input v-model="chatContext.compaction_enabled" type="checkbox" class="h-4 w-4 accent-brand" />
          Ativar compactador de conversa
        </label>

        <label class="block text-sm">
          <span class="mb-1 block text-gray-500 dark:text-slate-400">Limite maximo de mensagens antes de compactar</span>
          <input
            v-model.number="chatContext.max_messages"
            type="number"
            min="4"
            max="200"
            class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
          />
        </label>

        <label class="block text-sm">
          <span class="mb-1 block text-gray-500 dark:text-slate-400">Mensagens recentes mantidas completas</span>
          <input
            v-model.number="chatContext.keep_last_messages"
            type="number"
            min="2"
            :max="chatContext.max_messages"
            class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
          />
        </label>

        <label class="block text-sm">
          <span class="mb-1 block text-gray-500 dark:text-slate-400">Tamanho maximo do resumo compacto</span>
          <input
            v-model.number="chatContext.max_summary_chars"
            type="number"
            min="500"
            max="20000"
            step="500"
            class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
          />
        </label>

        <div class="flex items-center gap-3">
          <button class="rounded-md bg-brand px-4 py-2 font-medium text-white disabled:opacity-50" :disabled="saving" @click="saveChatContext">
            {{ saving ? 'Salvando...' : 'Salvar contexto' }}
          </button>
          <span v-if="saved" class="text-sm text-brand">Configuração salva.</span>
        </div>
      </div>
    </section>
  </AppShell>
</template>
