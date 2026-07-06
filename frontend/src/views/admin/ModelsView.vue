<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Box, Cpu, DollarSign, Hash, RefreshCw, Save, Wrench } from '@lucide/vue'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

interface ModelRow {
  id: string
  provider_id: string
  provider_name: string
  name: string
  context_window: number | null
  input_price: number
  output_price: number
  active: boolean
  created_at: string
  updated_at: string
  tools?: ModelTool[]
}

interface ModelTool {
  id: string
  name: string
  description: string
  tool_type?: string
  enabled: boolean
  assigned?: boolean
}

const models = ref<ModelRow[]>([])
const selectedId = ref('')
const modelTools = ref<ModelTool[]>([])
const selectedToolIds = ref<string[]>([])
const savingTools = ref(false)

const selectedModel = computed(() => models.value.find((model) => model.id === selectedId.value) ?? models.value[0])
const activeCount = computed(() => models.value.filter((model) => model.active).length)
const providerCount = computed(() => new Set(models.value.map((model) => model.provider_id)).size)

async function load() {
  const { data } = await http.get('/api/admin/models')
  models.value = Array.isArray(data) ? data : []
  if (!selectedId.value && models.value.length) selectedId.value = models.value[0].id
  if (selectedId.value) await loadModelTools(selectedId.value)
}

async function selectModel(model: ModelRow) {
  selectedId.value = model.id
  await loadModelTools(model.id)
}

async function loadModelTools(modelId: string) {
  const { data } = await http.get(`/api/admin/models/${modelId}/tools`)
  modelTools.value = Array.isArray(data) ? data : []
  selectedToolIds.value = modelTools.value.filter((tool) => tool.assigned).map((tool) => tool.id)
}

async function saveModelTools() {
  if (!selectedModel.value) return
  savingTools.value = true
  try {
    await http.put(`/api/admin/models/${selectedModel.value.id}/tools`, {
      tool_ids: selectedToolIds.value
    })
    await load()
  } finally {
    savingTools.value = false
  }
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('pt-BR', {
    dateStyle: 'short',
    timeStyle: 'short'
  }).format(new Date(value))
}

onMounted(load)
</script>

<template>
  <AppShell>
    <section class="p-6">
      <div class="mb-5 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold">Modelos</h1>
          <p class="mt-1 text-sm text-gray-500">Modelos cadastrados por provider para uso no chat.</p>
        </div>
        <button class="grid h-9 w-9 place-items-center rounded-md border border-gray-300 bg-white" title="Atualizar" @click="load">
          <RefreshCw class="h-4 w-4" />
        </button>
      </div>

      <div class="mb-5 grid grid-cols-3 gap-4">
        <div class="rounded-lg bg-white p-4 shadow-sm">
          <div class="text-sm text-gray-500">Total</div>
          <div class="mt-2 text-2xl font-semibold">{{ models.length }}</div>
        </div>
        <div class="rounded-lg bg-white p-4 shadow-sm">
          <div class="text-sm text-gray-500">Ativos</div>
          <div class="mt-2 text-2xl font-semibold">{{ activeCount }}</div>
        </div>
        <div class="rounded-lg bg-white p-4 shadow-sm">
          <div class="text-sm text-gray-500">Providers</div>
          <div class="mt-2 text-2xl font-semibold">{{ providerCount }}</div>
        </div>
      </div>

      <div class="grid grid-cols-[1fr_360px] gap-5">
        <div class="grid auto-rows-min grid-cols-2 gap-4">
          <button
            v-for="model in models"
            :key="model.id"
            class="rounded-lg border bg-white p-4 text-left shadow-sm transition hover:border-brand"
            :class="selectedId === model.id ? 'border-brand ring-1 ring-brand' : 'border-transparent'"
            @click="selectModel(model)"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <Cpu class="h-4 w-4 text-brand" />
                  <h2 class="truncate font-semibold">{{ model.name }}</h2>
                </div>
                <p class="mt-2 text-sm text-gray-500">{{ model.provider_name }}</p>
              </div>
              <span class="rounded px-2 py-0.5 text-xs" :class="model.active ? 'bg-emerald-50 text-emerald-700' : 'bg-gray-100 text-gray-500'">
                {{ model.active ? 'ativo' : 'inativo' }}
              </span>
            </div>
            <div v-if="model.tools?.length" class="mt-3 flex items-center gap-1 text-xs text-brand">
              <Wrench class="h-3.5 w-3.5" />
              {{ model.tools.length }} ferramenta{{ model.tools.length === 1 ? '' : 's' }}
            </div>
            <div class="mt-4 grid grid-cols-3 gap-2 text-xs">
              <div class="rounded-md bg-gray-50 p-2">
                <div class="text-gray-500">Contexto</div>
                <div class="mt-1 font-medium">{{ model.context_window ?? '-' }}</div>
              </div>
              <div class="rounded-md bg-gray-50 p-2">
                <div class="text-gray-500">Input</div>
                <div class="mt-1 font-medium">{{ model.input_price }}</div>
              </div>
              <div class="rounded-md bg-gray-50 p-2">
                <div class="text-gray-500">Output</div>
                <div class="mt-1 font-medium">{{ model.output_price }}</div>
              </div>
            </div>
          </button>

          <div v-if="!models.length" class="col-span-2 rounded-lg bg-white p-6 text-sm text-gray-500 shadow-sm">
            Nenhum modelo cadastrado ainda. Cadastre em Providers.
          </div>
        </div>

        <aside class="rounded-lg bg-white p-5 shadow-sm">
          <template v-if="selectedModel">
            <div class="mb-5 flex items-center gap-2">
              <Box class="h-5 w-5 text-brand" />
              <h2 class="text-lg font-semibold">Detalhes</h2>
            </div>

            <div class="space-y-4 text-sm">
              <div>
                <div class="text-gray-500">Nome</div>
                <div class="mt-1 break-words font-medium">{{ selectedModel.name }}</div>
              </div>
              <div>
                <div class="text-gray-500">Provider</div>
                <div class="mt-1 font-medium">{{ selectedModel.provider_name }}</div>
              </div>
              <div>
                <div class="text-gray-500">ID</div>
                <div class="mt-1 break-all font-mono text-xs">{{ selectedModel.id }}</div>
              </div>
              <div>
                <div class="text-gray-500">Provider ID</div>
                <div class="mt-1 break-all font-mono text-xs">{{ selectedModel.provider_id }}</div>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div class="rounded-md bg-gray-50 p-3">
                  <div class="flex items-center gap-1 text-gray-500"><Hash class="h-3.5 w-3.5" /> Janela</div>
                  <div class="mt-1 font-semibold">{{ selectedModel.context_window ?? 'não definido' }}</div>
                </div>
                <div class="rounded-md bg-gray-50 p-3">
                  <div class="flex items-center gap-1 text-gray-500"><DollarSign class="h-3.5 w-3.5" /> Preços</div>
                  <div class="mt-1 font-semibold">{{ selectedModel.input_price }} / {{ selectedModel.output_price }}</div>
                </div>
              </div>

              <div>
                <div class="text-gray-500">Criado em</div>
                <div class="mt-1">{{ formatDate(selectedModel.created_at) }}</div>
              </div>
              <div>
                <div class="text-gray-500">Atualizado em</div>
                <div class="mt-1">{{ formatDate(selectedModel.updated_at) }}</div>
              </div>

              <div class="border-t border-gray-200 pt-4 dark:border-slate-700">
                <div class="mb-3 flex items-center justify-between gap-3">
                  <div>
                    <div class="flex items-center gap-2 font-semibold">
                      <Wrench class="h-4 w-4 text-brand" />
                      Ferramentas
                    </div>
                    <p class="mt-1 text-xs text-gray-500">A IA so podera chamar as ferramentas marcadas neste modelo.</p>
                  </div>
                  <button
                    class="inline-flex items-center gap-2 rounded-md bg-brand px-3 py-2 text-xs font-semibold text-white disabled:opacity-60"
                    :disabled="savingTools"
                    @click="saveModelTools"
                  >
                    <Save class="h-3.5 w-3.5" />
                    Salvar
                  </button>
                </div>

                <div v-if="modelTools.length" class="space-y-2">
                  <label
                    v-for="tool in modelTools"
                    :key="tool.id"
                    class="flex cursor-pointer items-start gap-3 rounded-md border border-gray-200 bg-gray-50 p-3 text-sm dark:border-slate-700 dark:bg-slate-900"
                    :class="!tool.enabled ? 'opacity-60' : ''"
                  >
                    <input
                      v-model="selectedToolIds"
                      class="mt-1 h-4 w-4"
                      type="checkbox"
                      :value="tool.id"
                      :disabled="!tool.enabled"
                    />
                    <span class="min-w-0">
                      <span class="flex items-center gap-2 font-medium">
                        {{ tool.name }}
                        <span class="rounded bg-gray-100 px-2 py-0.5 text-xs text-gray-500 dark:bg-slate-800 dark:text-slate-300">
                          {{ tool.tool_type === 'abstract' ? 'abstrata' : 'fisica' }}
                        </span>
                      </span>
                      <span class="mt-1 block text-xs text-gray-500">{{ tool.description }}</span>
                    </span>
                  </label>
                </div>
                <p v-else class="rounded-md bg-gray-50 p-3 text-xs text-gray-500 dark:bg-slate-900">
                  Nenhuma ferramenta MCP cadastrada ainda.
                </p>
              </div>
            </div>
          </template>
          <p v-else class="text-sm text-gray-500">Selecione um modelo para ver os detalhes.</p>
        </aside>
      </div>
    </section>
  </AppShell>
</template>
