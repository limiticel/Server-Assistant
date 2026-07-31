<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { Braces, ChevronLeft, ChevronRight, GripVertical, Globe2, Layers3, Pencil, Plus, RefreshCw, Save, Search, Server, Trash2, X } from '@lucide/vue'
import ConfirmModal from '../../components/ConfirmModal.vue'
import MessageModal from '../../components/MessageModal.vue'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

interface McpTool {
  id: string
  name: string
  description: string
  tool_type: 'physical' | 'abstract'
  input_schema: Record<string, unknown>
  config: Record<string, unknown>
  response_schema: Record<string, unknown>
  enabled: boolean
}

type ToolPayload = {
  name: string
  description: string
  tool_type: 'physical' | 'abstract'
  input_schema: Record<string, unknown>
  config: Record<string, unknown>
  response_schema: Record<string, unknown>
  enabled: boolean
}

const tools = ref<McpTool[]>([])
const saving = ref(false)
const savingEdit = ref(false)
const deletingId = ref('')
const toolToDelete = ref<McpTool | null>(null)
const editingTool = ref<McpTool | null>(null)
const pendingInfraPayload = ref<ToolPayload | null>(null)
const pendingInfraEditPayload = ref<ToolPayload | null>(null)
const selectedType = ref<'physical' | 'abstract' | 'infra'>('physical')
const toolSearch = ref('')
const currentPage = ref(1)
const pageSize = ref(5)
const errorModal = reactive({
  open: false,
  title: 'Erro na ferramenta',
  message: ''
})

const base = reactive({
  name: '',
  description: '',
  enabled: true,
  inputSchemaText: '{\n  "type": "object",\n  "properties": {},\n  "required": []\n}'
})

const physical = reactive({
  method: 'GET',
  url: '',
  headersText: '{\n  "Content-Type": "application/json"\n}',
  responseSchemaText: '{\n  "type": "object",\n  "properties": {}\n}'
})

const abstractTool = reactive({
  executionMode: 'sequential',
  instructions: '',
  toolSequence: [] as string[],
  staticResponse: ''
})

const editForm = reactive({
  name: '',
  description: '',
  tool_type: 'physical' as 'physical' | 'abstract' | 'infra',
  enabled: true,
  inputSchemaText: '{}',
  configText: '{}',
  responseSchemaText: '{}'
})

const physicalTools = computed(() => tools.value.filter((tool) => tool.tool_type === 'physical' && !isInfraConfig(tool.config)))
const abstractTools = computed(() => tools.value.filter((tool) => tool.tool_type === 'abstract'))
const infraTools = computed(() => tools.value.filter((tool) => isInfraConfig(tool.config)))
const callableTools = computed(() => tools.value.filter((tool) => tool.enabled && tool.tool_type === 'physical'))
const availableSequenceTools = computed(() =>
  callableTools.value.filter((tool) => !abstractTool.toolSequence.includes(tool.name))
)
const filteredTools = computed(() => {
  const query = toolSearch.value.trim().toLowerCase()
  if (!query) return tools.value
  return tools.value.filter((tool) =>
    [
      tool.name,
      tool.description,
      tool.tool_type,
      toolKind(tool),
      typeof tool.config?.kind === 'string' ? tool.config.kind : ''
    ]
      .join(' ')
      .toLowerCase()
      .includes(query)
  )
})
const totalPages = computed(() => Math.max(1, Math.ceil(filteredTools.value.length / pageSize.value)))
const pageStart = computed(() => (currentPage.value - 1) * pageSize.value)
const pageEnd = computed(() => Math.min(pageStart.value + pageSize.value, filteredTools.value.length))
const paginatedTools = computed(() => filteredTools.value.slice(pageStart.value, pageEnd.value))
const selectedSequenceTool = ref('')
const draggedSequenceIndex = ref<number | null>(null)

function parseJson(text: string, label: string) {
  try {
    const value = JSON.parse(text || '{}')
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('object expected')
    return value
  } catch {
    throw new Error(`${label} precisa ser um JSON object valido.`)
  }
}

function parseJsonArray(text: string, label: string) {
  try {
    const value = JSON.parse(text || '[]')
    if (!Array.isArray(value)) throw new Error('array expected')
    return value
  } catch {
    throw new Error(`${label} precisa ser um JSON array valido.`)
  }
}

function showError(error: any, fallback: string) {
  errorModal.title = 'Nao foi possivel salvar'
  errorModal.message = error?.response?.data?.error ?? error.message ?? fallback
  errorModal.open = true
}

function isInfraConfig(config: Record<string, unknown>) {
  return config.kind === 'infra' || config.kind === 'ssh' || config.kind === 'infra_ssh'
}

async function load() {
  const { data } = await http.get('/api/admin/mcp-tools')
  tools.value = Array.isArray(data) ? data : []
  clampCurrentPage()
}

function resetForm() {
  base.name = ''
  base.description = ''
  base.enabled = true
  base.inputSchemaText = '{\n  "type": "object",\n  "properties": {},\n  "required": []\n}'
  physical.method = 'GET'
  physical.url = ''
  physical.headersText = '{\n  "Content-Type": "application/json"\n}'
  physical.responseSchemaText = '{\n  "type": "object",\n  "properties": {}\n}'
  abstractTool.executionMode = 'sequential'
  abstractTool.instructions = ''
  abstractTool.toolSequence = []
  selectedSequenceTool.value = ''
  abstractTool.staticResponse = ''
  if (selectedType.value === 'infra') {
    base.inputSchemaText = infraInputSchemaText()
  }
}

function buildCreatePayload(): ToolPayload {
  const input_schema = parseJson(base.inputSchemaText, 'Parametros') as Record<string, unknown>
  if (selectedType.value === 'physical') {
    return {
        name: base.name,
        description: base.description,
        tool_type: 'physical',
        input_schema,
        config: {
          kind: 'api',
          method: physical.method,
          url: physical.url,
          headers: parseJson(physical.headersText, 'Headers')
        },
        response_schema: parseJson(physical.responseSchemaText, 'Modelo de response'),
        enabled: base.enabled
      }
  }

  if (selectedType.value === 'infra') {
    return {
        name: base.name,
        description: base.description,
        tool_type: 'physical',
        input_schema,
        config: {
          kind: 'infra',
          access_level: 'full'
        },
        response_schema: {
          type: 'object',
          properties: {
            success: { type: 'boolean' },
            exit_code: { type: 'integer' },
            stdout: { type: 'string' },
            stderr: { type: 'string' }
          }
        },
        enabled: base.enabled
      }
  }

  return {
        name: base.name,
        description: base.description,
        tool_type: 'abstract',
        input_schema,
        config: {
          kind: 'abstract',
          execution_mode: abstractTool.executionMode,
          instructions: abstractTool.instructions,
          tool_sequence: abstractTool.toolSequence,
          static_response: abstractTool.staticResponse
        },
        response_schema: {},
        enabled: base.enabled
      }
}

async function persistTool(payload: ToolPayload) {
  await http.post('/api/admin/mcp-tools', payload)
  resetForm()
  await load()
}

async function saveTool() {
  saving.value = true
  try {
    const payload = buildCreatePayload()
    if (selectedType.value === 'infra') {
      pendingInfraPayload.value = payload
      return
    }
    await persistTool(payload)
  } catch (error: any) {
    showError(error, 'Erro ao salvar ferramenta.')
  } finally {
    if (!pendingInfraPayload.value) {
      saving.value = false
    }
  }
}

async function confirmInfraCreate() {
  const payload = pendingInfraPayload.value
  if (!payload) return
  try {
    await persistTool(payload)
  } catch (error: any) {
    showError(error, 'Erro ao salvar ferramenta.')
  } finally {
    pendingInfraPayload.value = null
    saving.value = false
  }
}

function cancelInfraCreate() {
  pendingInfraPayload.value = null
  saving.value = false
}

function selectToolType(type: 'physical' | 'abstract' | 'infra') {
  selectedType.value = type
  if (type === 'infra') {
    base.inputSchemaText = infraInputSchemaText()
    if (!base.description.trim()) {
      base.description = 'Executa acoes de infraestrutura com acesso total conforme os parametros definidos.'
    }
  } else if (base.inputSchemaText === infraInputSchemaText()) {
    base.inputSchemaText = '{\n  "type": "object",\n  "properties": {},\n  "required": []\n}'
  }
}

function infraInputSchemaText() {
  return JSON.stringify(
    {
      type: 'object',
      properties: {
        runtime: {
          type: 'string',
          description: 'Tipo de execucao. Use local para maquina local ou ssh para servidor remoto.',
          enum: ['local', 'ssh']
        },
        command: {
          type: 'string',
          description: 'Comando de linha unica para executar.'
        },
        host: {
          type: 'string',
          description: 'Host quando runtime for ssh.'
        },
        port: {
          type: 'integer',
          default: 22
        },
        username: {
          type: 'string',
          description: 'Usuario quando runtime for ssh.'
        },
        password: {
          type: 'string',
          description: 'Senha opcional quando runtime for ssh.'
        },
        private_key_path: {
          type: 'string',
          description: 'Caminho opcional da chave privada quando runtime for ssh.'
        },
        timeout_seconds: {
          type: 'integer',
          minimum: 1,
          maximum: 120,
          default: 30
        }
      },
      required: ['runtime', 'command']
    },
    null,
    2
  )
}

function addSequenceTool() {
  const name = selectedSequenceTool.value
  if (!name || abstractTool.toolSequence.includes(name)) return
  abstractTool.toolSequence.push(name)
  selectedSequenceTool.value = ''
}

function removeSequenceTool(index: number) {
  abstractTool.toolSequence.splice(index, 1)
}

function moveSequenceTool(fromIndex: number, toIndex: number) {
  if (fromIndex === toIndex) return
  if (fromIndex < 0 || toIndex < 0) return
  if (fromIndex >= abstractTool.toolSequence.length || toIndex >= abstractTool.toolSequence.length) return

  const [item] = abstractTool.toolSequence.splice(fromIndex, 1)
  abstractTool.toolSequence.splice(toIndex, 0, item)
}

function startSequenceDrag(index: number) {
  draggedSequenceIndex.value = index
}

function dropSequenceTool(index: number) {
  if (draggedSequenceIndex.value === null) return
  moveSequenceTool(draggedSequenceIndex.value, index)
  draggedSequenceIndex.value = null
}

function toolKind(tool: McpTool) {
  const kind = typeof tool.config?.kind === 'string' ? tool.config.kind : ''
  if (kind === 'infra' || kind === 'ssh' || kind === 'infra_ssh') return 'infra'
  if (tool.tool_type === 'abstract') return 'abstrata'
  return 'fisica'
}

function startEditTool(tool: McpTool) {
  editingTool.value = tool
  editForm.name = tool.name
  editForm.description = tool.description
  editForm.tool_type = isInfraConfig(tool.config) ? 'infra' : tool.tool_type
  editForm.enabled = tool.enabled
  editForm.inputSchemaText = JSON.stringify(tool.input_schema, null, 2)
  editForm.configText = JSON.stringify(tool.config, null, 2)
  editForm.responseSchemaText = JSON.stringify(tool.response_schema, null, 2)
}

function cancelEditTool() {
  editingTool.value = null
}

async function saveEditedTool() {
  const tool = editingTool.value
  if (!tool) return
  savingEdit.value = true
  try {
    const payload = {
      name: editForm.name,
      description: editForm.description,
      tool_type: editForm.tool_type === 'infra' ? 'physical' : editForm.tool_type,
      input_schema: parseJson(editForm.inputSchemaText, 'Parametros'),
      config: editForm.tool_type === 'infra'
        ? { kind: 'infra', access_level: 'full' }
        : parseJson(editForm.configText, 'Config'),
      response_schema: parseJson(editForm.responseSchemaText, 'Modelo de response'),
      enabled: editForm.enabled
    } as ToolPayload
    if (isInfraConfig(payload.config)) {
      pendingInfraEditPayload.value = payload
      return
    }
    await persistEditedTool(tool.id, payload)
  } catch (error: any) {
    showError(error, 'Erro ao editar ferramenta.')
  } finally {
    if (!pendingInfraEditPayload.value) {
      savingEdit.value = false
    }
  }
}

async function persistEditedTool(id: string, payload: ToolPayload) {
  await http.put(`/api/admin/mcp-tools/${id}`, payload)
  editingTool.value = null
  await load()
}

async function confirmInfraEdit() {
  const tool = editingTool.value
  const payload = pendingInfraEditPayload.value
  if (!tool || !payload) return
  try {
    await persistEditedTool(tool.id, payload)
  } catch (error: any) {
    showError(error, 'Erro ao editar ferramenta.')
  } finally {
    pendingInfraEditPayload.value = null
    savingEdit.value = false
  }
}

function cancelInfraEdit() {
  pendingInfraEditPayload.value = null
  savingEdit.value = false
}

function clampCurrentPage() {
  currentPage.value = Math.min(Math.max(1, currentPage.value), totalPages.value)
}

function goToPage(page: number) {
  currentPage.value = Math.min(Math.max(1, page), totalPages.value)
}

function displayJson(value: Record<string, unknown>) {
  return JSON.stringify(maskSecrets(value), null, 2)
}

function maskSecrets(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(maskSecrets)
  if (!value || typeof value !== 'object') return value

  return Object.fromEntries(
    Object.entries(value as Record<string, unknown>).map(([key, entry]) => [
      key,
      ['password', 'senha', 'secret', 'token', 'api_key', 'authorization'].includes(key.toLowerCase()) ? '********' : maskSecrets(entry)
    ])
  )
}

function requestDeleteTool(tool: McpTool) {
  toolToDelete.value = tool
}

async function confirmDeleteTool() {
  const tool = toolToDelete.value
  if (!tool) return
  deletingId.value = tool.id
  try {
    await http.delete(`/api/admin/mcp-tools/${tool.id}`)
    await load()
  } finally {
    deletingId.value = ''
    toolToDelete.value = null
  }
}

watch([toolSearch, pageSize], () => {
  currentPage.value = 1
})

watch(totalPages, clampCurrentPage)

onMounted(load)
</script>

<template>
  <AppShell>
    <section class="p-6">
      <div class="mb-5 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold">MCP</h1>
          <p class="mt-1 text-sm text-gray-500">Crie ferramentas fisicas de API e ferramentas abstratas orquestradas.</p>
        </div>
        <button class="grid h-9 w-9 place-items-center rounded-md border border-gray-300 bg-white" title="Atualizar" @click="load">
          <RefreshCw class="h-4 w-4" />
        </button>
      </div>

      <div class="grid grid-cols-[460px_1fr] gap-5">
        <form class="space-y-4 rounded-lg bg-white p-5 shadow-sm" @submit.prevent="saveTool">
          <div class="grid grid-cols-3 gap-2">
            <button
              type="button"
              class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm"
              :class="selectedType === 'physical' ? 'border-brand bg-brand text-white' : 'border-gray-300'"
              @click="selectToolType('physical')"
            >
              <Globe2 class="h-4 w-4" />
              Fisica API
            </button>
            <button
              type="button"
              class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm"
              :class="selectedType === 'abstract' ? 'border-brand bg-brand text-white' : 'border-gray-300'"
              @click="selectToolType('abstract')"
            >
              <Layers3 class="h-4 w-4" />
              Abstrata
            </button>
            <button
              type="button"
              class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm"
              :class="selectedType === 'infra' ? 'border-brand bg-brand text-white' : 'border-gray-300'"
              @click="selectToolType('infra')"
            >
              <Server class="h-4 w-4" />
              Infra
            </button>
          </div>

          <input v-model="base.name" class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Nome da tool" />
          <textarea v-model="base.description" class="min-h-20 w-full resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Descricao da tool" />

          <div>
            <label class="mb-1 block text-xs font-medium text-gray-500">Parametros esperados</label>
            <textarea v-model="base.inputSchemaText" class="h-36 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
          </div>

          <template v-if="selectedType === 'physical'">
            <div class="grid grid-cols-[110px_1fr] gap-2">
              <select v-model="physical.method" class="rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
                <option>GET</option>
                <option>POST</option>
                <option>PUT</option>
                <option>PATCH</option>
                <option>DELETE</option>
              </select>
              <input v-model="physical.url" class="rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="URL da API" />
            </div>
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Headers</label>
              <textarea v-model="physical.headersText" class="h-24 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
            </div>
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Modelo de response</label>
              <textarea v-model="physical.responseSchemaText" class="h-28 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
            </div>
          </template>

          <template v-else-if="selectedType === 'abstract'">
            <select v-model="abstractTool.executionMode" class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
              <option value="sequential">Chamar ferramentas em ordem</option>
              <option value="parallel">Chamar ferramentas sem ordem</option>
              <option value="text">Responder com texto/instrucao</option>
            </select>
            <textarea v-model="abstractTool.instructions" class="min-h-28 w-full resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Instrucoes da ferramenta abstrata" />
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Ferramentas chamadas</label>
              <div class="rounded-md border border-gray-300 bg-white p-3">
                <div class="flex gap-2">
                  <select v-model="selectedSequenceTool" class="min-w-0 flex-1 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
                    <option value="">Selecionar ferramenta</option>
                    <option v-for="tool in availableSequenceTools" :key="tool.id" :value="tool.name">
                      {{ tool.name }}
                    </option>
                  </select>
                  <button
                    class="inline-flex items-center gap-2 rounded-md border border-gray-300 px-3 py-2 text-sm font-medium hover:bg-gray-50 disabled:opacity-50"
                    type="button"
                    :disabled="!selectedSequenceTool"
                    @click="addSequenceTool"
                  >
                    <Plus class="h-4 w-4" />
                    Adicionar
                  </button>
                </div>

                <div v-if="abstractTool.toolSequence.length" class="mt-3 space-y-2">
                  <div
                    v-for="(toolName, index) in abstractTool.toolSequence"
                    :key="toolName"
                    class="flex items-center gap-2 rounded-md border border-gray-200 bg-gray-50 px-2 py-2 text-sm"
                    draggable="true"
                    @dragstart="startSequenceDrag(index)"
                    @dragover.prevent
                    @drop="dropSequenceTool(index)"
                  >
                    <GripVertical class="h-4 w-4 shrink-0 cursor-grab text-gray-400" />
                    <span class="grid h-6 w-6 shrink-0 place-items-center rounded bg-white text-xs font-semibold text-gray-500">{{ index + 1 }}</span>
                    <span class="min-w-0 flex-1 truncate font-medium">{{ toolName }}</span>
                    <button
                      class="grid h-7 w-7 shrink-0 place-items-center rounded-md text-gray-400 hover:bg-white hover:text-red-600"
                      type="button"
                      title="Remover da sequencia"
                      @click="removeSequenceTool(index)"
                    >
                      <X class="h-4 w-4" />
                    </button>
                  </div>
                </div>

                <div v-else class="mt-3 rounded-md border border-dashed border-gray-300 p-4 text-center text-xs text-gray-500">
                  Nenhuma ferramenta adicionada.
                </div>
              </div>
            </div>
            <textarea v-model="abstractTool.staticResponse" class="min-h-20 w-full resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Texto retornado em casos especificos" />
          </template>

          <label class="flex items-center gap-2 text-sm">
            <input v-model="base.enabled" type="checkbox" />
            Ativa
          </label>

          <button class="flex w-full items-center justify-center gap-2 rounded-md bg-brand px-3 py-2 text-sm font-medium text-white disabled:opacity-50" :disabled="saving">
            <Plus class="h-4 w-4" />
            Criar ferramenta
          </button>
        </form>

        <div class="space-y-5">
          <div class="grid grid-cols-4 gap-3">
            <div class="rounded-lg bg-white p-4 shadow-sm">
              <div class="text-xs text-gray-500">Total</div>
              <div class="mt-1 text-2xl font-semibold">{{ tools.length }}</div>
            </div>
            <div class="rounded-lg bg-white p-4 shadow-sm">
              <div class="text-xs text-gray-500">Fisicas</div>
              <div class="mt-1 text-2xl font-semibold">{{ physicalTools.length }}</div>
            </div>
            <div class="rounded-lg bg-white p-4 shadow-sm">
              <div class="text-xs text-gray-500">Abstratas</div>
              <div class="mt-1 text-2xl font-semibold">{{ abstractTools.length }}</div>
            </div>
            <div class="rounded-lg bg-white p-4 shadow-sm">
              <div class="text-xs text-gray-500">Infra</div>
              <div class="mt-1 text-2xl font-semibold">{{ infraTools.length }}</div>
            </div>
          </div>

          <div class="rounded-lg bg-white p-4 shadow-sm dark:bg-slate-900">
            <div class="flex flex-wrap items-center justify-between gap-3">
              <div class="flex min-w-[240px] flex-1 items-center gap-2 rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700">
                <Search class="h-4 w-4 text-gray-500" />
                <input v-model="toolSearch" class="w-full border-0 bg-transparent p-0 text-sm outline-none" placeholder="Buscar por nome, descricao ou tipo" />
              </div>
              <div class="flex items-center gap-2 text-sm">
                <span class="text-gray-500">Por pagina</span>
                <select v-model.number="pageSize" class="rounded-md border border-gray-300 bg-white px-2 py-2 text-sm dark:border-slate-700 dark:bg-slate-950">
                  <option :value="3">3</option>
                  <option :value="5">5</option>
                  <option :value="10">10</option>
                </select>
              </div>
            </div>
            <div class="mt-3 flex items-center justify-between text-xs text-gray-500">
              <span>
                {{ filteredTools.length ? pageStart + 1 : 0 }}-{{ pageEnd }} de {{ filteredTools.length }} ferramenta{{ filteredTools.length === 1 ? '' : 's' }}
              </span>
              <span v-if="toolSearch">Filtro ativo</span>
            </div>
          </div>

          <div class="grid gap-3">
            <article v-for="tool in paginatedTools" :key="tool.id" class="rounded-lg bg-white p-4 shadow-sm">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <Braces class="h-4 w-4 text-brand" />
                    <h2 class="font-semibold">{{ tool.name }}</h2>
                    <span class="rounded bg-gray-100 px-2 py-0.5 text-xs">{{ toolKind(tool) }}</span>
                    <span class="rounded px-2 py-0.5 text-xs" :class="tool.enabled ? 'bg-emerald-50 text-emerald-700' : 'bg-gray-100 text-gray-500'">{{ tool.enabled ? 'ativa' : 'inativa' }}</span>
                  </div>
                  <p class="mt-2 text-sm text-gray-600">{{ tool.description }}</p>
                  <div class="mt-3 grid grid-cols-2 gap-3 text-xs">
                    <pre class="max-h-32 overflow-auto rounded-md bg-gray-50 p-3">{{ displayJson(tool.input_schema) }}</pre>
                    <pre class="max-h-32 overflow-auto rounded-md bg-gray-50 p-3">{{ displayJson(tool.config) }}</pre>
                  </div>
                </div>
                <div class="flex shrink-0 gap-2">
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md border border-gray-300 text-gray-500 hover:bg-gray-50 hover:text-brand dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
                    title="Editar ferramenta"
                    @click="startEditTool(tool)"
                  >
                    <Pencil class="h-4 w-4" />
                  </button>
                  <button
                    class="grid h-8 w-8 place-items-center rounded-md border border-red-200 text-red-600 hover:bg-red-50 disabled:opacity-50"
                    title="Excluir ferramenta"
                    :disabled="deletingId === tool.id"
                    @click="requestDeleteTool(tool)"
                  >
                    <Trash2 class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </article>
            <div v-if="!paginatedTools.length" class="rounded-lg border border-dashed border-gray-300 bg-white p-8 text-center text-sm text-gray-500 shadow-sm dark:border-slate-700 dark:bg-slate-900">
              Nenhuma ferramenta encontrada.
            </div>
          </div>

          <div v-if="filteredTools.length" class="flex flex-wrap items-center justify-between gap-3 rounded-lg bg-white p-3 shadow-sm dark:bg-slate-900">
            <button class="inline-flex items-center gap-2 rounded-md border border-gray-300 px-3 py-2 text-sm disabled:opacity-50 dark:border-slate-700" :disabled="currentPage <= 1" @click="goToPage(currentPage - 1)">
              <ChevronLeft class="h-4 w-4" />
              Anterior
            </button>
            <div class="text-sm text-gray-500">Pagina {{ currentPage }} de {{ totalPages }}</div>
            <button class="inline-flex items-center gap-2 rounded-md border border-gray-300 px-3 py-2 text-sm disabled:opacity-50 dark:border-slate-700" :disabled="currentPage >= totalPages" @click="goToPage(currentPage + 1)">
              Proxima
              <ChevronRight class="h-4 w-4" />
            </button>
          </div>
        </div>
      </div>
    </section>
    <ConfirmModal
      :open="Boolean(toolToDelete)"
      title="Excluir ferramenta"
      :message="`A ferramenta '${toolToDelete?.name ?? ''}' sera removida.`"
      confirm-label="Excluir"
      tone="danger"
      @cancel="toolToDelete = null"
      @confirm="confirmDeleteTool"
    />
    <ConfirmModal
      :open="Boolean(pendingInfraPayload)"
      title="Liberar acesso total de Infra?"
      message="Esta ferramenta permite que a IA execute comandos via SSH no servidor configurado. Isso pode alterar arquivos, instalar pacotes, parar servicos, expor dados ou causar indisponibilidade. Continue apenas se esta maquina e credencial forem confiaveis."
      confirm-label="Liberar acesso"
      cancel-label="Cancelar"
      tone="danger"
      @cancel="cancelInfraCreate"
      @confirm="confirmInfraCreate"
    />
    <ConfirmModal
      :open="Boolean(pendingInfraEditPayload)"
      title="Confirmar alteracao de Infra?"
      message="Esta alteracao mantem ou concede acesso SSH para a IA executar comandos no servidor configurado. Revise host, usuario, credenciais e se a ferramenta deve ficar ativa antes de continuar."
      confirm-label="Salvar mesmo assim"
      cancel-label="Cancelar"
      tone="danger"
      @cancel="cancelInfraEdit"
      @confirm="confirmInfraEdit"
    />
    <MessageModal
      :open="errorModal.open"
      :title="errorModal.title"
      :message="errorModal.message"
      tone="danger"
      action-label="Entendi"
      @close="errorModal.open = false"
    />
    <div v-if="editingTool" class="fixed inset-0 z-50 grid place-items-center bg-black/50 p-4">
      <form class="max-h-[90vh] w-full max-w-3xl overflow-auto rounded-lg bg-white p-5 shadow-xl dark:bg-slate-900" @submit.prevent="saveEditedTool">
        <div class="mb-4 flex items-center justify-between gap-3">
          <div>
            <h2 class="text-lg font-semibold">Editar ferramenta</h2>
            <p class="mt-1 text-sm text-gray-500">Para SSH, coloque usuario e senha dentro de Config.</p>
          </div>
          <button class="grid h-8 w-8 place-items-center rounded-md text-gray-500 hover:bg-gray-100 dark:hover:bg-slate-800" type="button" title="Fechar" @click="cancelEditTool">
            <X class="h-4 w-4" />
          </button>
        </div>

        <div class="grid gap-4">
          <div class="grid grid-cols-[1fr_160px] gap-3">
            <input v-model="editForm.name" class="rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Nome da tool" />
            <select v-model="editForm.tool_type" class="rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
              <option value="physical">Fisica</option>
              <option value="infra">Infra</option>
              <option value="abstract">Abstrata</option>
            </select>
          </div>
          <textarea v-model="editForm.description" class="min-h-20 resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Descricao" />

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Parametros esperados</label>
              <textarea v-model="editForm.inputSchemaText" class="h-56 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
            </div>
            <div v-if="editForm.tool_type !== 'infra'">
              <label class="mb-1 block text-xs font-medium text-gray-500">Config</label>
              <textarea v-model="editForm.configText" class="h-56 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
              <p v-if="editingTool.name === 'ubuntu_server_ssh'" class="mt-2 text-xs text-gray-500">
                Exemplo: {"{"}"kind":"ssh","host":"127.0.0.1","port":2222,"username":"usuario","password":"sua-senha"{"}"}
              </p>
            </div>
            <div v-else class="rounded-md border border-amber-200 bg-amber-50 p-4 text-sm text-amber-900 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-100">
              Infra usa apenas os parametros esperados para definir runtime, comando, host, credenciais e demais entradas. Ao salvar, a config fixa sera marcada como acesso total.
            </div>
          </div>

          <div>
            <label class="mb-1 block text-xs font-medium text-gray-500">Modelo de response</label>
            <textarea v-model="editForm.responseSchemaText" class="h-32 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
          </div>

          <label class="flex items-center gap-2 text-sm">
            <input v-model="editForm.enabled" type="checkbox" />
            Ativa
          </label>

          <div class="flex justify-end gap-2">
            <button class="rounded-md border border-gray-300 px-4 py-2 text-sm" type="button" @click="cancelEditTool">Cancelar</button>
            <button class="inline-flex items-center gap-2 rounded-md bg-brand px-4 py-2 text-sm font-semibold text-white disabled:opacity-60" :disabled="savingEdit">
              <Save class="h-4 w-4" />
              Salvar
            </button>
          </div>
        </div>
      </form>
    </div>
  </AppShell>
</template>
