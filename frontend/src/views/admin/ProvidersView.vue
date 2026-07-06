<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { Plus, RefreshCw, Trash2 } from '@lucide/vue'
import ConfirmModal from '../../components/ConfirmModal.vue'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

interface ProviderRow {
  id: string
  name: string
  base_url: string
  provider_type: string
  default_model?: string
  active: boolean
  has_api_key: boolean
}

interface ModelRow {
  id: string
  provider_id: string
  provider_name: string
  name: string
  active: boolean
}

const providers = ref<ProviderRow[]>([])
const models = ref<ModelRow[]>([])
const saving = ref(false)
const deletingId = ref('')
const providerToDelete = ref<ProviderRow | null>(null)
const form = reactive({
  name: 'OpenAI',
  provider_type: 'openai_compatible',
  base_url: 'https://api.openai.com/v1',
  api_key: '',
  default_model: 'gpt-4o-mini',
  active: true
})

const modelForm = reactive({
  provider_id: '',
  name: ''
})

async function load() {
  const [providersResponse, modelsResponse] = await Promise.all([
    http.get('/api/admin/providers'),
    http.get('/api/admin/models')
  ])
  providers.value = providersResponse.data
  models.value = modelsResponse.data
  if (!modelForm.provider_id && providers.value.length) modelForm.provider_id = providers.value[0].id
}

function preset(type: string) {
  form.provider_type = type
  if (type === 'openai') {
    form.name = 'OpenAI'
    form.base_url = 'https://api.openai.com/v1'
    form.default_model = 'gpt-4o-mini'
  }
  if (type === 'deepseek') {
    form.name = 'DeepSeek'
    form.base_url = 'https://api.deepseek.com/v1'
    form.default_model = 'deepseek-chat'
  }
  if (type === 'ollama') {
    form.name = 'Ollama'
    form.base_url = 'http://localhost:11434/v1'
    form.default_model = 'llama3.1:8b'
    form.api_key = ''
  }
  if (type === 'openai_compatible') {
    form.name = 'OpenAI Compatível'
    form.base_url = ''
    form.default_model = ''
  }
}

async function saveProvider() {
  saving.value = true
  try {
    await http.post('/api/admin/providers', form)
    form.api_key = ''
    await load()
  } finally {
    saving.value = false
  }
}

async function saveModel() {
  if (!modelForm.provider_id || !modelForm.name.trim()) return
  await http.post('/api/admin/models', { ...modelForm, active: true })
  modelForm.name = ''
  await load()
}

function requestDeleteProvider(provider: ProviderRow) {
  providerToDelete.value = provider
}

async function confirmDeleteProvider() {
  const provider = providerToDelete.value
  if (!provider) return
  deletingId.value = provider.id
  try {
    await http.delete(`/api/admin/providers/${provider.id}`)
    if (modelForm.provider_id === provider.id) modelForm.provider_id = ''
    await load()
  } finally {
    deletingId.value = ''
    providerToDelete.value = null
  }
}

onMounted(load)
</script>

<template>
  <AppShell>
    <section class="p-6">
      <div class="mb-5 flex items-center justify-between">
        <h1 class="text-2xl font-semibold">Providers</h1>
        <button class="grid h-9 w-9 place-items-center rounded-md border border-gray-300 bg-white" title="Atualizar" @click="load">
          <RefreshCw class="h-4 w-4" />
        </button>
      </div>

      <div class="grid grid-cols-[420px_1fr] gap-5">
        <form class="space-y-3 rounded-lg bg-white p-5 shadow-sm" @submit.prevent="saveProvider">
          <div class="grid grid-cols-2 gap-2">
            <button type="button" class="rounded-md border px-3 py-2 text-sm" @click="preset('openai')">OpenAI</button>
            <button type="button" class="rounded-md border px-3 py-2 text-sm" @click="preset('deepseek')">DeepSeek</button>
            <button type="button" class="rounded-md border px-3 py-2 text-sm" @click="preset('ollama')">Ollama</button>
            <button type="button" class="rounded-md border px-3 py-2 text-sm" @click="preset('openai_compatible')">Compatível</button>
          </div>
          <input v-model="form.name" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Nome" />
          <select v-model="form.provider_type" class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
            <option value="openai">OpenAI</option>
            <option value="ollama">Ollama</option>
            <option value="anthropic">Anthropic</option>
            <option value="gemini">Gemini</option>
            <option value="deepseek">DeepSeek</option>
            <option value="openai_compatible">Compatível OpenAI</option>
          </select>
          <input v-model="form.base_url" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Base URL" />
          <input v-model="form.api_key" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="API Key" type="password" />
          <input v-model="form.default_model" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Modelo padrão" />
          <label class="flex items-center gap-2 text-sm">
            <input v-model="form.active" type="checkbox" />
            Ativo
          </label>
          <button class="flex w-full items-center justify-center gap-2 rounded-md bg-brand px-3 py-2 text-sm font-medium text-white disabled:opacity-50" :disabled="saving">
            <Plus class="h-4 w-4" />
            Salvar provider
          </button>
        </form>

        <div class="space-y-5">
          <div class="rounded-lg bg-white p-5 shadow-sm">
            <h2 class="mb-3 font-semibold">Modelos</h2>
            <form class="flex gap-2" @submit.prevent="saveModel">
              <select v-model="modelForm.provider_id" class="min-w-44 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
                <option v-for="provider in providers" :key="provider.id" :value="provider.id">{{ provider.name }}</option>
              </select>
              <input v-model="modelForm.name" class="flex-1 rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Nome do modelo" />
              <button class="rounded-md bg-ink px-3 py-2 text-sm font-medium text-white">Adicionar</button>
            </form>
          </div>

          <div class="overflow-auto rounded-lg bg-white shadow-sm">
            <table class="w-full text-left text-sm">
              <thead class="border-b border-gray-100 text-gray-500">
                <tr>
                  <th class="p-3">Nome</th>
                  <th class="p-3">Tipo</th>
                  <th class="p-3">Modelo</th>
                  <th class="p-3">Key</th>
                  <th class="p-3">Ativo</th>
                  <th class="w-12 p-3"></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="provider in providers" :key="provider.id" class="border-b border-gray-100">
                  <td class="p-3">
                    <div class="font-medium">{{ provider.name }}</div>
                    <div class="max-w-md truncate text-xs text-gray-500">{{ provider.base_url }}</div>
                  </td>
                  <td class="p-3">{{ provider.provider_type }}</td>
                  <td class="p-3">{{ provider.default_model }}</td>
                  <td class="p-3">{{ provider.has_api_key ? 'configurada' : 'vazia' }}</td>
                  <td class="p-3">{{ provider.active ? 'sim' : 'não' }}</td>
                  <td class="p-3">
                    <button
                      class="grid h-8 w-8 place-items-center rounded-md border border-red-200 text-red-600 hover:bg-red-50 disabled:opacity-50"
                      title="Excluir provider"
                      :disabled="deletingId === provider.id"
                      @click="requestDeleteProvider(provider)"
                    >
                      <Trash2 class="h-4 w-4" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>

          <div class="overflow-auto rounded-lg bg-white shadow-sm">
            <table class="w-full text-left text-sm">
              <tbody>
                <tr v-for="model in models" :key="model.id" class="border-b border-gray-100">
                  <td class="p-3 font-medium">{{ model.name }}</td>
                  <td class="p-3 text-gray-500">{{ model.provider_name }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </section>
    <ConfirmModal
      :open="Boolean(providerToDelete)"
      title="Excluir provider"
      :message="`O provider '${providerToDelete?.name ?? ''}' e seus modelos vinculados serao removidos.`"
      confirm-label="Excluir"
      tone="danger"
      @cancel="providerToDelete = null"
      @confirm="confirmDeleteProvider"
    />
  </AppShell>
</template>
