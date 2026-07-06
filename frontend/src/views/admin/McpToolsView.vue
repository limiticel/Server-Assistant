<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { Braces, Globe2, Layers3, Pencil, Plus, RefreshCw, Save, Trash2, X } from '@lucide/vue'
import ConfirmModal from '../../components/ConfirmModal.vue'
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

const tools = ref<McpTool[]>([])
const saving = ref(false)
const savingEdit = ref(false)
const deletingId = ref('')
const toolToDelete = ref<McpTool | null>(null)
const editingTool = ref<McpTool | null>(null)
const selectedType = ref<'physical' | 'abstract'>('physical')

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
  toolSequenceText: '[]',
  staticResponse: ''
})

const editForm = reactive({
  name: '',
  description: '',
  tool_type: 'physical' as 'physical' | 'abstract',
  enabled: true,
  inputSchemaText: '{}',
  configText: '{}',
  responseSchemaText: '{}'
})

const physicalTools = computed(() => tools.value.filter((tool) => tool.tool_type === 'physical'))
const abstractTools = computed(() => tools.value.filter((tool) => tool.tool_type === 'abstract'))

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

async function load() {
  const { data } = await http.get('/api/admin/mcp-tools')
  tools.value = Array.isArray(data) ? data : []
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
  abstractTool.toolSequenceText = '[]'
  abstractTool.staticResponse = ''
}

async function saveTool() {
  saving.value = true
  try {
    const input_schema = parseJson(base.inputSchemaText, 'Parametros')
    const payload =
      selectedType.value === 'physical'
        ? {
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
        : {
            name: base.name,
            description: base.description,
            tool_type: 'abstract',
            input_schema,
            config: {
              kind: 'abstract',
              execution_mode: abstractTool.executionMode,
              instructions: abstractTool.instructions,
              tool_sequence: parseJsonArray(abstractTool.toolSequenceText, 'Sequencia de ferramentas'),
              static_response: abstractTool.staticResponse
            },
            response_schema: {},
            enabled: base.enabled
          }

    await http.post('/api/admin/mcp-tools', payload)
    resetForm()
    await load()
  } catch (error: any) {
    window.alert(error?.response?.data?.error ?? error.message ?? 'Erro ao salvar ferramenta.')
  } finally {
    saving.value = false
  }
}

function startEditTool(tool: McpTool) {
  editingTool.value = tool
  editForm.name = tool.name
  editForm.description = tool.description
  editForm.tool_type = tool.tool_type
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
    await http.put(`/api/admin/mcp-tools/${tool.id}`, {
      name: editForm.name,
      description: editForm.description,
      tool_type: editForm.tool_type,
      input_schema: parseJson(editForm.inputSchemaText, 'Parametros'),
      config: parseJson(editForm.configText, 'Config'),
      response_schema: parseJson(editForm.responseSchemaText, 'Modelo de response'),
      enabled: editForm.enabled
    })
    editingTool.value = null
    await load()
  } catch (error: any) {
    window.alert(error?.response?.data?.error ?? error.message ?? 'Erro ao editar ferramenta.')
  } finally {
    savingEdit.value = false
  }
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
          <div class="grid grid-cols-2 gap-2">
            <button
              type="button"
              class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm"
              :class="selectedType === 'physical' ? 'border-brand bg-brand text-white' : 'border-gray-300'"
              @click="selectedType = 'physical'"
            >
              <Globe2 class="h-4 w-4" />
              Fisica API
            </button>
            <button
              type="button"
              class="flex items-center justify-center gap-2 rounded-md border px-3 py-2 text-sm"
              :class="selectedType === 'abstract' ? 'border-brand bg-brand text-white' : 'border-gray-300'"
              @click="selectedType = 'abstract'"
            >
              <Layers3 class="h-4 w-4" />
              Abstrata
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

          <template v-else>
            <select v-model="abstractTool.executionMode" class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm">
              <option value="sequential">Chamar ferramentas em ordem</option>
              <option value="parallel">Chamar ferramentas sem ordem</option>
              <option value="text">Responder com texto/instrucao</option>
            </select>
            <textarea v-model="abstractTool.instructions" class="min-h-28 w-full resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Instrucoes da ferramenta abstrata" />
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Ferramentas chamadas</label>
              <textarea v-model="abstractTool.toolSequenceText" class="h-24 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
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
          <div class="grid grid-cols-3 gap-3">
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
          </div>

          <div class="grid gap-3">
            <article v-for="tool in tools" :key="tool.id" class="rounded-lg bg-white p-4 shadow-sm">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <Braces class="h-4 w-4 text-brand" />
                    <h2 class="font-semibold">{{ tool.name }}</h2>
                    <span class="rounded bg-gray-100 px-2 py-0.5 text-xs">{{ tool.tool_type === 'physical' ? 'fisica' : 'abstrata' }}</span>
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
              <option value="abstract">Abstrata</option>
            </select>
          </div>
          <textarea v-model="editForm.description" class="min-h-20 resize-none rounded-md border border-gray-300 px-3 py-2 text-sm" placeholder="Descricao" />

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Parametros esperados</label>
              <textarea v-model="editForm.inputSchemaText" class="h-56 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
            </div>
            <div>
              <label class="mb-1 block text-xs font-medium text-gray-500">Config</label>
              <textarea v-model="editForm.configText" class="h-56 w-full resize-none rounded-md border border-gray-300 p-3 font-mono text-xs" spellcheck="false" />
              <p v-if="editingTool.name === 'ubuntu_server_ssh'" class="mt-2 text-xs text-gray-500">
                Exemplo: {"{"}"kind":"ssh","host":"127.0.0.1","port":2222,"username":"usuario","password":"sua-senha"{"}"}
              </p>
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
