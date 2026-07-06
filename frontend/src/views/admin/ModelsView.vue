<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { Box, Cpu, DollarSign, Hash, Pencil, RefreshCw, Save, Sparkles, Wrench, X } from '@lucide/vue'
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
  assistant_name?: string | null
  personality?: string | null
  temperament?: string | null
  pre_prompt?: string | null
  pre_prompt_limit?: number
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
const savingPersona = ref(false)
const personaSaved = ref(false)
const personaModalOpen = ref(false)
const editingPersonaModelId = ref('')
const personaForm = reactive({
  assistant_name: '',
  personality: '',
  temperament: '',
  pre_prompt: '',
  pre_prompt_limit: 2000
})

const selectedModel = computed(() => models.value.find((model) => model.id === selectedId.value) ?? models.value[0])
const editingPersonaModel = computed(() => models.value.find((model) => model.id === editingPersonaModelId.value) ?? selectedModel.value)
const activeCount = computed(() => models.value.filter((model) => model.active).length)
const providerCount = computed(() => new Set(models.value.map((model) => model.provider_id)).size)
const prePromptUsed = computed(() => personaForm.pre_prompt.length)
const prePromptOverLimit = computed(() => prePromptUsed.value > Number(personaForm.pre_prompt_limit || 0))
const selectedModelHasPersona = computed(() =>
  Boolean(selectedModel.value?.assistant_name || selectedModel.value?.personality || selectedModel.value?.temperament || selectedModel.value?.pre_prompt)
)

async function load() {
  const { data } = await http.get('/api/admin/models')
  models.value = Array.isArray(data) ? data : []
  if (!selectedId.value && models.value.length) selectedId.value = models.value[0].id
  syncPersonaForm()
  if (selectedId.value) await loadModelTools(selectedId.value)
}

async function selectModel(model: ModelRow) {
  selectedId.value = model.id
  syncPersonaForm()
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

async function saveModelPersona() {
  const model = editingPersonaModel.value
  if (!model || prePromptOverLimit.value) return
  savingPersona.value = true
  personaSaved.value = false
  try {
    await http.put(`/api/admin/models/${model.id}/persona`, {
      assistant_name: personaForm.assistant_name,
      personality: personaForm.personality,
      temperament: personaForm.temperament,
      pre_prompt: personaForm.pre_prompt,
      pre_prompt_limit: Number(personaForm.pre_prompt_limit)
    })
    await load()
    personaSaved.value = true
    personaModalOpen.value = false
  } finally {
    savingPersona.value = false
  }
}

function syncPersonaForm(model = selectedModel.value) {
  personaForm.assistant_name = model?.assistant_name ?? ''
  personaForm.personality = model?.personality ?? ''
  personaForm.temperament = model?.temperament ?? ''
  personaForm.pre_prompt = model?.pre_prompt ?? ''
  personaForm.pre_prompt_limit = model?.pre_prompt_limit ?? 2000
  personaSaved.value = false
}

function openPersonaModal(model: ModelRow) {
  editingPersonaModelId.value = model.id
  syncPersonaForm(model)
  personaModalOpen.value = true
}

function closePersonaModal() {
  personaModalOpen.value = false
  savingPersona.value = false
  personaSaved.value = false
}

function formatDate(value: string) {
  return new Intl.DateTimeFormat('pt-BR', {
    dateStyle: 'short',
    timeStyle: 'short'
  }).format(new Date(value))
}

watch(selectedId, () => syncPersonaForm())
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
            class="group relative rounded-lg border bg-white p-4 text-left shadow-sm transition hover:border-brand"
            :class="selectedId === model.id ? 'border-brand ring-1 ring-brand' : 'border-transparent'"
            @click="selectModel(model)"
          >
            <span
              class="absolute right-3 top-12 grid h-8 w-8 place-items-center rounded-md border border-gray-200 bg-white text-gray-500 opacity-0 transition hover:border-brand hover:text-brand group-hover:opacity-100 dark:border-slate-700 dark:bg-slate-950"
              title="Editar persona"
              @click.stop="openPersonaModal(model)"
            >
              <Pencil class="h-4 w-4" />
            </span>
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
            <div v-if="model.assistant_name || model.personality || model.temperament || model.pre_prompt" class="mt-2 flex items-center gap-1 text-xs text-brand">
              <Sparkles class="h-3.5 w-3.5" />
              persona configurada
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
                  <div class="flex items-center gap-2 font-semibold">
                    <Sparkles class="h-4 w-4 text-brand" />
                    Persona
                  </div>
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md border border-gray-300 text-gray-500 hover:border-brand hover:text-brand dark:border-slate-700"
                    title="Editar persona"
                    @click="openPersonaModal(selectedModel)"
                  >
                    <Pencil class="h-4 w-4" />
                  </button>
                </div>

                <div v-if="selectedModelHasPersona" class="space-y-2 rounded-md bg-gray-50 p-3 text-xs dark:bg-slate-900">
                  <div v-if="selectedModel.assistant_name">
                    <span class="text-gray-500">Nome:</span>
                    <span class="ml-1 font-medium">{{ selectedModel.assistant_name }}</span>
                  </div>
                  <div v-if="selectedModel.personality" class="line-clamp-2">
                    <span class="text-gray-500">Personalidade:</span>
                    <span class="ml-1">{{ selectedModel.personality }}</span>
                  </div>
                  <div v-if="selectedModel.temperament" class="line-clamp-2">
                    <span class="text-gray-500">Temperamento:</span>
                    <span class="ml-1">{{ selectedModel.temperament }}</span>
                  </div>
                  <div v-if="selectedModel.pre_prompt">
                    <span class="text-gray-500">Pre-prompt:</span>
                    <span class="ml-1">{{ selectedModel.pre_prompt.length }} caracteres</span>
                  </div>
                </div>
                <p v-else class="rounded-md bg-gray-50 p-3 text-xs text-gray-500 dark:bg-slate-900">
                  Nenhuma persona configurada.
                </p>
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

    <Teleport to="body">
      <div v-if="personaModalOpen" class="fixed inset-0 z-50 grid place-items-center bg-black/55 p-4" @click.self="closePersonaModal">
        <section class="flex max-h-[90vh] w-full max-w-2xl flex-col overflow-hidden rounded-lg border border-gray-200 bg-white text-ink shadow-xl dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100">
          <header class="flex shrink-0 items-center justify-between border-b border-gray-200 px-5 py-4 dark:border-slate-700">
            <div>
              <div class="flex items-center gap-2 text-lg font-semibold">
                <Sparkles class="h-5 w-5 text-brand" />
                Persona do modelo
              </div>
              <p class="mt-1 text-sm text-gray-500 dark:text-slate-400">
                {{ editingPersonaModel?.name }} · {{ editingPersonaModel?.provider_name }}
              </p>
            </div>
            <button class="grid h-9 w-9 place-items-center rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-slate-800" title="Fechar" @click="closePersonaModal">
              <X class="h-4 w-4" />
            </button>
          </header>

          <div class="min-h-0 flex-1 space-y-4 overflow-auto p-5">
            <label class="block">
              <span class="mb-1 block text-sm text-gray-500 dark:text-slate-400">Nome da IA</span>
              <input
                v-model="personaForm.assistant_name"
                maxlength="80"
                class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
                placeholder="Ex: Aurora"
              />
            </label>

            <div class="grid gap-4 md:grid-cols-2">
              <label class="block">
                <span class="mb-1 block text-sm text-gray-500 dark:text-slate-400">Personalidade</span>
                <textarea
                  v-model="personaForm.personality"
                  maxlength="600"
                  class="h-28 w-full resize-none rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
                  placeholder="Ex: curiosa, direta, cuidadosa e colaborativa."
                />
              </label>

              <label class="block">
                <span class="mb-1 block text-sm text-gray-500 dark:text-slate-400">Temperamento</span>
                <textarea
                  v-model="personaForm.temperament"
                  maxlength="600"
                  class="h-28 w-full resize-none rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
                  placeholder="Ex: calmo, confiante, bem-humorado sem exagero."
                />
              </label>
            </div>

            <div class="grid gap-4 md:grid-cols-[1fr_130px]">
              <label class="block">
                <span class="mb-1 block text-sm text-gray-500 dark:text-slate-400">Pre-prompt</span>
                <textarea
                  v-model="personaForm.pre_prompt"
                  class="h-44 w-full resize-none rounded-md border bg-white px-3 py-2 text-sm dark:bg-slate-950"
                  :class="prePromptOverLimit ? 'border-red-500' : 'border-gray-300 dark:border-slate-700'"
                  placeholder="Instrucoes fixas que este modelo deve seguir no chat."
                />
              </label>
              <label class="block">
                <span class="mb-1 block text-sm text-gray-500 dark:text-slate-400">Limite</span>
                <input
                  v-model.number="personaForm.pre_prompt_limit"
                  type="number"
                  min="200"
                  max="12000"
                  step="100"
                  class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm dark:border-slate-700 dark:bg-slate-950"
                />
              </label>
            </div>

            <div class="flex items-center justify-between text-xs">
              <span :class="prePromptOverLimit ? 'text-red-500' : 'text-gray-500'">
                {{ prePromptUsed }} / {{ personaForm.pre_prompt_limit }} caracteres
              </span>
              <span v-if="prePromptOverLimit" class="text-red-500">Reduza o pre-prompt ou aumente o limite.</span>
            </div>
          </div>

          <footer class="flex shrink-0 justify-end gap-2 border-t border-gray-200 px-5 py-4 dark:border-slate-700">
            <button class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium hover:bg-gray-100 dark:border-slate-600 dark:hover:bg-slate-800" @click="closePersonaModal">
              Cancelar
            </button>
            <button
              class="inline-flex items-center gap-2 rounded-md bg-brand px-4 py-2 text-sm font-semibold text-white disabled:opacity-60"
              :disabled="savingPersona || prePromptOverLimit"
              @click="saveModelPersona"
            >
              <Save class="h-4 w-4" />
              {{ savingPersona ? 'Salvando...' : 'Salvar persona' }}
            </button>
          </footer>
        </section>
      </div>
    </Teleport>
  </AppShell>
</template>
