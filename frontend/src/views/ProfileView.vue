<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Brain, CalendarClock, RefreshCw, Save } from '@lucide/vue'
import { http } from '../api/http'
import AppShell from '../layouts/AppShell.vue'
import { applyTheme, getSavedTheme, type ThemeMode } from '../theme'

interface AiProvider {
  id: string
  name: string
  provider_type: string
  default_model?: string
  active: boolean
}

interface AiModel {
  id: string
  provider_id: string
  name: string
  active: boolean
}

const theme = ref<ThemeMode>(getSavedTheme())
const profileName = ref('')
const profileEmail = ref('')
const profileSaving = ref(false)
const profileSaved = ref(false)
const profileError = ref('')
const contextSaving = ref(false)
const contextSaved = ref(false)
const summarySaving = ref(false)
const summaryGenerating = ref(false)
const summarySaved = ref(false)
const summaryError = ref('')
const providers = ref<AiProvider[]>([])
const models = ref<AiModel[]>([])
const chatContext = ref({
  compaction_enabled: true,
  max_messages: 80,
  keep_last_messages: 24,
  max_summary_chars: 8000
})
const profileSummary = ref({
  settings: {
    enabled: false,
    provider: '',
    model: ''
  },
  summary: '',
  generated_at: ''
})

const modelsForSummaryProvider = computed(() =>
  models.value.filter((model) => model.provider_id === profileSummary.value.settings.provider && model.active)
)

const generatedAtLabel = computed(() => {
  if (!profileSummary.value.generated_at) return 'Ainda nao gerado'
  return new Intl.DateTimeFormat('pt-BR', {
    dateStyle: 'short',
    timeStyle: 'short'
  }).format(new Date(profileSummary.value.generated_at))
})

const generatedToday = computed(() => {
  if (!profileSummary.value.generated_at) return false
  const generated = new Date(profileSummary.value.generated_at)
  const now = new Date()
  return generated.toDateString() === now.toDateString()
})

onMounted(async () => {
  const [profileResponse, chatContextResponse, summaryResponse, providersResponse, modelsResponse] = await Promise.all([
    http.get('/api/auth/me'),
    http.get('/api/settings/chat-context'),
    http.get('/api/settings/profile-summary'),
    http.get('/api/admin/providers'),
    http.get('/api/admin/models')
  ])
  profileName.value = profileResponse.data.name ?? ''
  profileEmail.value = profileResponse.data.email ?? ''
  chatContext.value = chatContextResponse.data
  profileSummary.value = normalizeProfileSummary(summaryResponse.data)
  providers.value = providersResponse.data.filter((provider: AiProvider) => provider.active)
  models.value = modelsResponse.data.filter((model: AiModel) => model.active)
  ensureProfileSummarySelection()

  if (profileSummary.value.settings.enabled && !generatedToday.value) {
    await generateProfileSummary()
  }
})

watch(
  () => profileSummary.value.settings.provider,
  () => ensureProfileSummarySelection()
)

async function saveProfile() {
  profileSaving.value = true
  profileSaved.value = false
  profileError.value = ''

  try {
    const { data } = await http.put('/api/auth/me', {
      name: profileName.value.trim()
    })
    profileName.value = data.name ?? profileName.value
    profileEmail.value = data.email ?? profileEmail.value
    applyTheme(theme.value)
    profileSaved.value = true
  } catch (error: any) {
    profileError.value = error?.response?.data?.error ?? 'Nao foi possivel salvar o perfil.'
  } finally {
    profileSaving.value = false
  }
}

async function saveChatContext() {
  contextSaving.value = true
  contextSaved.value = false
  try {
    const { data } = await http.put('/api/settings/chat-context', {
      compaction_enabled: chatContext.value.compaction_enabled,
      max_messages: Number(chatContext.value.max_messages),
      keep_last_messages: Number(chatContext.value.keep_last_messages),
      max_summary_chars: Number(chatContext.value.max_summary_chars)
    })
    chatContext.value = data
    contextSaved.value = true
  } finally {
    contextSaving.value = false
  }
}

async function saveProfileSummarySettings() {
  summarySaving.value = true
  summarySaved.value = false
  summaryError.value = ''

  try {
    const { data } = await http.put('/api/settings/profile-summary', profileSummary.value.settings)
    profileSummary.value = normalizeProfileSummary(data)
    summarySaved.value = true
    return true
  } catch (error: any) {
    summaryError.value = error?.response?.data?.error ?? 'Nao foi possivel salvar o resumo diario.'
    return false
  } finally {
    summarySaving.value = false
  }
}

async function generateProfileSummary() {
  summaryGenerating.value = true
  summarySaved.value = false
  summaryError.value = ''

  try {
    const savedSettings = await saveProfileSummarySettings()
    if (!savedSettings) return
    const { data } = await http.post('/api/settings/profile-summary/generate')
    profileSummary.value = normalizeProfileSummary(data)
  } catch (error: any) {
    summaryError.value = error?.response?.data?.error ?? 'Nao foi possivel gerar o resumo diario.'
  } finally {
    summaryGenerating.value = false
  }
}

function ensureProfileSummarySelection() {
  if (!profileSummary.value.settings.provider && providers.value.length) {
    const preferred = providers.value.find((provider) => provider.provider_type.toLowerCase() === 'ollama') ?? providers.value[0]
    profileSummary.value.settings.provider = preferred.id
  }

  if (!modelsForSummaryProvider.value.some((model) => model.name === profileSummary.value.settings.model)) {
    const provider = providers.value.find((item) => item.id === profileSummary.value.settings.provider)
    profileSummary.value.settings.model =
      provider?.default_model ||
      modelsForSummaryProvider.value[0]?.name ||
      ''
  }
}

function normalizeProfileSummary(data: any) {
  return {
    settings: {
      enabled: Boolean(data?.settings?.enabled),
      provider: data?.settings?.provider ?? '',
      model: data?.settings?.model ?? ''
    },
    summary: data?.summary ?? '',
    generated_at: data?.generated_at ?? ''
  }
}
</script>

<template>
  <AppShell>
    <section class="mx-auto max-w-7xl space-y-6 p-6">
      <div>
        <h1 class="text-2xl font-semibold">Perfil</h1>
        <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">Preferencias pessoais, memoria diaria e contexto enviado para a IA.</p>
      </div>

      <div class="grid items-start gap-6 xl:grid-cols-[minmax(360px,480px)_minmax(520px,1fr)]">
        <div class="space-y-6">
          <div class="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
            <div class="border-b border-gray-100 px-5 py-4 dark:border-slate-800">
              <h2 class="text-lg font-semibold">Conta</h2>
              <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">Dados basicos e aparencia do sistema.</p>
            </div>
            <div class="space-y-4 p-5">
              <label class="block text-sm">
                <span class="mb-1 block text-gray-500 dark:text-slate-400">Nome</span>
                <input v-model="profileName" class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950" placeholder="Nome" />
              </label>
              <label class="block text-sm">
                <span class="mb-1 block text-gray-500 dark:text-slate-400">Email</span>
                <input :value="profileEmail" class="w-full rounded-md border border-gray-200 bg-gray-50 px-3 py-2 text-gray-500 dark:border-slate-700 dark:bg-slate-950/60 dark:text-slate-400" disabled />
              </label>
              <label class="block text-sm">
                <span class="mb-1 block text-gray-500 dark:text-slate-400">Tema</span>
                <select v-model="theme" class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950">
                  <option value="light">Tema claro</option>
                  <option value="dark">Tema escuro</option>
                </select>
              </label>
              <div v-if="profileError" class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900/60 dark:bg-red-950/30 dark:text-red-200">
                {{ profileError }}
              </div>
              <div class="flex items-center gap-3">
                <button class="rounded-md bg-brand px-4 py-2 font-medium text-white disabled:opacity-50" :disabled="profileSaving" @click="saveProfile">
                  {{ profileSaving ? 'Salvando...' : 'Salvar perfil' }}
                </button>
                <span v-if="profileSaved" class="text-sm text-brand">Perfil salvo.</span>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
            <div class="border-b border-gray-100 px-5 py-4 dark:border-slate-800">
              <h2 class="text-lg font-semibold">Contexto do chat</h2>
              <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">
                Compacta mensagens antigas mantendo as recentes completas.
              </p>
            </div>
            <div class="space-y-4 p-5">
              <label class="flex items-center justify-between gap-4 rounded-md border border-gray-200 p-3 text-sm dark:border-slate-700">
                <span>Ativar compactador de conversa</span>
                <input v-model="chatContext.compaction_enabled" type="checkbox" class="h-4 w-4 accent-brand" />
              </label>

              <div class="grid gap-3 sm:grid-cols-3 xl:grid-cols-1 2xl:grid-cols-3">
                <label class="block text-sm">
                  <span class="mb-1 block text-gray-500 dark:text-slate-400">Limite</span>
                  <input
                    v-model.number="chatContext.max_messages"
                    type="number"
                    min="4"
                    max="200"
                    class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
                  />
                </label>

                <label class="block text-sm">
                  <span class="mb-1 block text-gray-500 dark:text-slate-400">Recentes</span>
                  <input
                    v-model.number="chatContext.keep_last_messages"
                    type="number"
                    min="2"
                    :max="chatContext.max_messages"
                    class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
                  />
                </label>

                <label class="block text-sm">
                  <span class="mb-1 block text-gray-500 dark:text-slate-400">Resumo max.</span>
                  <input
                    v-model.number="chatContext.max_summary_chars"
                    type="number"
                    min="500"
                    max="20000"
                    step="500"
                    class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
                  />
                </label>
              </div>

              <div class="flex items-center gap-3">
                <button class="rounded-md bg-brand px-4 py-2 font-medium text-white disabled:opacity-50" :disabled="contextSaving" @click="saveChatContext">
                  {{ contextSaving ? 'Salvando...' : 'Salvar contexto' }}
                </button>
                <span v-if="contextSaved" class="text-sm text-brand">Configuracao salva.</span>
              </div>
            </div>
          </div>
        </div>

        <div class="rounded-lg border border-gray-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
          <div class="flex items-start gap-3 border-b border-gray-100 px-5 py-4 dark:border-slate-800">
            <div class="grid h-10 w-10 shrink-0 place-items-center rounded-md bg-brand text-white">
              <Brain class="h-5 w-5" />
            </div>
            <div class="min-w-0">
              <h2 class="text-lg font-semibold">Resumo diario do usuario</h2>
              <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">
                Uma IA escolhida por voce resume o contexto recente uma vez por dia.
              </p>
            </div>
          </div>
          <div class="space-y-5 p-5">
            <div class="grid gap-4 lg:grid-cols-[220px_1fr]">
              <label class="flex items-center justify-between gap-4 rounded-md border border-gray-200 p-3 text-sm dark:border-slate-700">
                <span>Ativar resumo diario</span>
                <input v-model="profileSummary.settings.enabled" type="checkbox" class="h-4 w-4 accent-brand" />
              </label>

              <div class="grid gap-3 sm:grid-cols-2">
                <label class="block text-sm">
                  <span class="mb-1 block text-gray-500 dark:text-slate-400">Provider</span>
                  <select
                    v-model="profileSummary.settings.provider"
                    class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
                  >
                    <option v-for="provider in providers" :key="provider.id" :value="provider.id">
                      {{ provider.name }}
                    </option>
                  </select>
                </label>

                <label class="block text-sm">
                  <span class="mb-1 block text-gray-500 dark:text-slate-400">Modelo</span>
                  <select
                    v-model="profileSummary.settings.model"
                    class="w-full rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700 dark:bg-slate-950"
                  >
                    <option v-for="model in modelsForSummaryProvider" :key="model.id" :value="model.name">
                      {{ model.name }}
                    </option>
                  </select>
                </label>
              </div>
            </div>

            <div class="rounded-md border border-gray-200 bg-gray-50 p-4 dark:border-slate-700 dark:bg-slate-950/50">
              <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <div class="flex min-w-0 items-center gap-2 text-sm text-gray-500 dark:text-slate-400">
                  <CalendarClock class="h-4 w-4 shrink-0" />
                  <span>{{ generatedAtLabel }}</span>
                </div>
                <span
                  class="rounded-full px-2 py-1 text-xs font-semibold"
                  :class="generatedToday ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-200' : 'bg-gray-200 text-gray-600 dark:bg-slate-800 dark:text-slate-300'"
                >
                  {{ generatedToday ? 'Resumo de hoje' : 'Pendente hoje' }}
                </span>
              </div>
              <p v-if="profileSummary.summary" class="whitespace-pre-line text-sm leading-6 text-gray-700 dark:text-slate-200">
                {{ profileSummary.summary }}
              </p>
              <p v-else class="text-sm text-gray-500 dark:text-slate-400">
                Nenhum resumo gerado ainda.
              </p>
            </div>

            <div v-if="summaryError" class="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900/60 dark:bg-red-950/30 dark:text-red-200">
              {{ summaryError }}
            </div>

            <div class="flex flex-wrap items-center gap-3">
              <button
                class="inline-flex items-center gap-2 rounded-md bg-brand px-4 py-2 font-medium text-white disabled:opacity-50"
                :disabled="summarySaving || summaryGenerating"
                @click="saveProfileSummarySettings"
              >
                <Save class="h-4 w-4" />
                {{ summarySaving ? 'Salvando...' : 'Salvar IA' }}
              </button>
              <button
                class="inline-flex items-center gap-2 rounded-md border border-gray-300 px-4 py-2 font-medium text-gray-700 hover:bg-gray-50 disabled:opacity-50 dark:border-slate-700 dark:text-slate-100 dark:hover:bg-slate-800"
                :disabled="summaryGenerating || !profileSummary.settings.provider || !profileSummary.settings.model"
                @click="generateProfileSummary"
              >
                <RefreshCw class="h-4 w-4" :class="summaryGenerating ? 'animate-spin' : ''" />
                {{ summaryGenerating ? 'Gerando...' : 'Gerar resumo do dia' }}
              </button>
              <span v-if="summarySaved" class="text-sm text-brand">Configuracao salva.</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  </AppShell>
</template>
