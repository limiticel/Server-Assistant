<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { AlertTriangle, BarChart3, Bot, Database, MessageSquareText, RefreshCw, Search, Users } from '@lucide/vue'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

interface ProviderUsage {
  provider: string
  provider_type: string
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
  estimated_cost: string
  estimated_cost_brl?: string | null
}

interface DashboardStats {
  general?: {
    users: number
    conversations: number
    messages: number
    configured_env_providers: string[]
  }
  billable_usage?: {
    prompt_tokens: number
    completion_tokens: number
    total_tokens: number
    estimated_cost: string
    estimated_cost_usd?: string
    estimated_cost_brl?: string | null
    currency?: string
    source_currency?: string
    exchange_rate?: number
    exchange_rate_date?: string
    exchange_rate_source?: string
    by_provider: ProviderUsage[]
  }
}

const stats = ref<DashboardStats>({})
const loading = ref(false)
const error = ref('')
const activeView = ref<'overview' | 'providers'>('overview')
const providerFilter = ref('')

const providerRows = computed(() => stats.value.billable_usage?.by_provider ?? [])
const maxProviderTokens = computed(() => Math.max(...providerRows.value.map((row) => row.total_tokens), 1))
const filteredProviders = computed(() => {
  const filter = providerFilter.value.trim().toLowerCase()
  if (!filter) return providerRows.value
  return providerRows.value.filter((row) => `${row.provider} ${row.provider_type}`.toLowerCase().includes(filter))
})

const generalCards = computed(() => [
  { label: 'Usuarios', value: stats.value.general?.users ?? 0, icon: Users, tone: 'text-cyan-300', hint: 'Contas cadastradas' },
  { label: 'Conversas', value: stats.value.general?.conversations ?? 0, icon: MessageSquareText, tone: 'text-emerald-300', hint: 'Historico criado' },
  { label: 'Mensagens', value: stats.value.general?.messages ?? 0, icon: Bot, tone: 'text-amber-300', hint: 'Entradas e respostas' }
])

const billableCards = computed(() => [
  { label: 'Prompt', value: stats.value.billable_usage?.prompt_tokens ?? 0, hint: 'Tokens enviados' },
  { label: 'Completion', value: stats.value.billable_usage?.completion_tokens ?? 0, hint: 'Tokens gerados' },
  { label: 'Total', value: stats.value.billable_usage?.total_tokens ?? 0, hint: 'Prompt + completion' },
  {
    label: 'Custo',
    value: formatCost(stats.value.billable_usage?.estimated_cost_brl, stats.value.billable_usage?.estimated_cost_usd),
    hint: stats.value.billable_usage?.estimated_cost_brl
      ? `Estimativa em BRL (${formatUSD(stats.value.billable_usage?.estimated_cost_usd ?? 0)})`
      : 'Estimativa em USD'
  }
])

const totalTokens = computed(() => stats.value.billable_usage?.total_tokens ?? 0)
const promptShare = computed(() => percent(stats.value.billable_usage?.prompt_tokens ?? 0, totalTokens.value))
const completionShare = computed(() => percent(stats.value.billable_usage?.completion_tokens ?? 0, totalTokens.value))
const exchangeRateLabel = computed(() => {
  const rate = stats.value.billable_usage?.exchange_rate
  if (!rate) return 'Cotacao USD/BRL indisponivel; exibindo custo em USD.'

  const date = stats.value.billable_usage?.exchange_rate_date
  const source = stats.value.billable_usage?.exchange_rate_source ?? 'API de cambio'
  return `Cotacao ${source}: US$1 = ${formatBRL(rate)}${date ? ` em ${date}` : ''}.`
})

async function load() {
  loading.value = true
  error.value = ''
  try {
    const { data } = await http.get('/api/admin/dashboard')
    stats.value = data
  } catch (err: any) {
    error.value = err?.response?.data?.error ?? err.message ?? 'Nao foi possivel carregar o dashboard.'
  } finally {
    loading.value = false
  }
}

function formatNumber(value: number) {
  return new Intl.NumberFormat('pt-BR').format(value)
}

function formatUSD(value: string | number) {
  const numeric = Number(value)
  if (!Number.isFinite(numeric)) return 'US$0.000000'
  return `US$${numeric.toFixed(6)}`
}

function formatBRL(value: string | number) {
  const numeric = Number(value)
  if (!Number.isFinite(numeric)) return 'R$0,00'
  return new Intl.NumberFormat('pt-BR', {
    style: 'currency',
    currency: 'BRL',
    minimumFractionDigits: 6,
    maximumFractionDigits: 6
  }).format(numeric)
}

function formatCost(valueBrl?: string | null, valueUsd?: string | number) {
  if (valueBrl) return formatBRL(valueBrl)
  return formatUSD(valueUsd ?? 0)
}

function percent(value: number, total: number) {
  if (!total) return 0
  return Math.round((value / total) * 100)
}

function providerWidth(value: number) {
  return `${Math.max(4, Math.round((value / maxProviderTokens.value) * 100))}%`
}

onMounted(load)
</script>

<template>
  <AppShell>
    <section class="p-6">
      <div class="mb-6 flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 class="text-2xl font-semibold">Dashboard</h1>
          <p class="mt-1 text-sm text-gray-500">Uso, tokens e providers em operacao.</p>
        </div>
        <button class="inline-flex items-center gap-2 rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium hover:bg-gray-100 disabled:opacity-60 dark:border-slate-700 dark:bg-slate-900 dark:hover:bg-slate-800" :disabled="loading" @click="load">
          <RefreshCw class="h-4 w-4" :class="loading ? 'animate-spin' : ''" />
          Atualizar
        </button>
      </div>

      <div v-if="error" class="mb-5 rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700 dark:border-red-900 dark:bg-red-950/40 dark:text-red-200">
        {{ error }}
      </div>

      <div class="mb-6 grid gap-4 lg:grid-cols-[1.2fr_0.8fr]">
        <section class="rounded-lg border border-gray-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
          <div class="mb-5 flex items-center justify-between gap-3">
            <div>
              <div class="text-xs font-semibold uppercase text-gray-500">Uso geral</div>
              <div class="mt-1 text-sm text-gray-500">Movimento registrado na plataforma.</div>
            </div>
            <Database class="h-5 w-5 text-brand" />
          </div>
          <div class="grid gap-3 sm:grid-cols-3">
            <div v-for="card in generalCards" :key="card.label" class="rounded-md border border-gray-200 bg-gray-50 p-4 dark:border-slate-800 dark:bg-slate-950">
              <component :is="card.icon" class="h-5 w-5" :class="card.tone" />
              <div class="mt-4 text-sm text-gray-500">{{ card.label }}</div>
              <div class="mt-1 text-3xl font-semibold">{{ formatNumber(card.value) }}</div>
              <div class="mt-1 text-xs text-gray-500">{{ card.hint }}</div>
            </div>
          </div>
        </section>

        <section class="rounded-lg border border-amber-400/40 bg-amber-50 p-5 text-amber-950 shadow-sm dark:border-amber-700/60 dark:bg-amber-950/30 dark:text-amber-100">
          <div class="flex items-start gap-3">
            <AlertTriangle class="mt-0.5 h-5 w-5 shrink-0" />
            <div>
              <h2 class="text-sm font-semibold">Custo estimado parcial</h2>
              <p class="mt-2 text-sm leading-6">
                Custos da OpenAI sao calculados em USD e convertidos para real usando cotacao atual da API Frankfurter. Providers locais continuam fora do calculo financeiro.
              </p>
            </div>
          </div>
        </section>
      </div>

      <section class="mb-6 rounded-lg border border-gray-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
          <div>
            <div class="text-xs font-semibold uppercase text-gray-500">Uso contabilizado</div>
            <div class="mt-1 text-sm text-gray-500">Distribuicao dos tokens com custo rastreado.</div>
            <div class="mt-1 text-xs text-gray-500">{{ exchangeRateLabel }}</div>
          </div>
          <div class="inline-flex rounded-md border border-gray-300 bg-gray-50 p-1 text-sm dark:border-slate-700 dark:bg-slate-950">
            <button class="rounded px-3 py-1.5" :class="activeView === 'overview' ? 'bg-white text-ink shadow-sm dark:bg-slate-800 dark:text-slate-100' : 'text-gray-500'" @click="activeView = 'overview'">Resumo</button>
            <button class="rounded px-3 py-1.5" :class="activeView === 'providers' ? 'bg-white text-ink shadow-sm dark:bg-slate-800 dark:text-slate-100' : 'text-gray-500'" @click="activeView = 'providers'">Providers</button>
          </div>
        </div>

        <template v-if="activeView === 'overview'">
          <div class="grid gap-3 md:grid-cols-4">
            <div v-for="card in billableCards" :key="card.label" class="rounded-md border border-gray-200 bg-gray-50 p-4 dark:border-slate-800 dark:bg-slate-950">
              <div class="text-sm text-gray-500">{{ card.label }}</div>
              <div class="mt-2 text-2xl font-semibold">{{ typeof card.value === 'number' ? formatNumber(card.value) : card.value }}</div>
              <div class="mt-1 text-xs text-gray-500">{{ card.hint }}</div>
            </div>
          </div>

          <div class="mt-5">
            <div class="mb-2 flex justify-between text-xs text-gray-500">
              <span>Prompt {{ promptShare }}%</span>
              <span>Completion {{ completionShare }}%</span>
            </div>
            <div class="flex h-3 overflow-hidden rounded-full bg-gray-100 dark:bg-slate-950">
              <div class="bg-cyan-400" :style="{ width: `${promptShare}%` }" />
              <div class="bg-emerald-400" :style="{ width: `${completionShare}%` }" />
            </div>
          </div>
        </template>

        <template v-else>
          <div class="mb-4 flex items-center gap-2 rounded-md border border-gray-300 px-3 py-2 dark:border-slate-700">
            <Search class="h-4 w-4 text-gray-500" />
            <input v-model="providerFilter" class="w-full border-0 bg-transparent p-0 text-sm outline-none" placeholder="Filtrar provider" />
          </div>

          <div class="grid gap-3">
            <article v-for="row in filteredProviders" :key="`${row.provider}-${row.provider_type}`" class="rounded-md border border-gray-200 bg-gray-50 p-4 dark:border-slate-800 dark:bg-slate-950">
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <div class="font-semibold">{{ row.provider }}</div>
                  <div class="mt-1 text-xs uppercase text-gray-500">{{ row.provider_type }}</div>
                </div>
                <div class="text-right">
                  <div class="font-semibold">{{ formatNumber(row.total_tokens) }}</div>
                  <div class="text-xs text-gray-500">
                    {{ formatCost(row.estimated_cost_brl, row.estimated_cost) }}
                    <span v-if="row.estimated_cost_brl" class="ml-1">({{ formatUSD(row.estimated_cost) }})</span>
                  </div>
                </div>
              </div>
              <div class="mt-4 h-2 overflow-hidden rounded-full bg-gray-200 dark:bg-slate-800">
                <div class="h-full rounded-full bg-brand" :style="{ width: providerWidth(row.total_tokens) }" />
              </div>
              <div class="mt-3 grid grid-cols-2 gap-3 text-xs text-gray-500">
                <div>Prompt: <span class="font-semibold text-ink dark:text-slate-100">{{ formatNumber(row.prompt_tokens) }}</span></div>
                <div>Completion: <span class="font-semibold text-ink dark:text-slate-100">{{ formatNumber(row.completion_tokens) }}</span></div>
              </div>
            </article>

            <div v-if="!filteredProviders.length" class="rounded-md border border-dashed border-gray-300 p-6 text-center text-sm text-gray-500 dark:border-slate-700">
              Nenhum provider encontrado.
            </div>
          </div>
        </template>
      </section>

      <section class="rounded-lg border border-gray-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <div class="mb-3 flex items-center gap-2">
          <BarChart3 class="h-4 w-4 text-brand" />
          <h2 class="text-sm font-semibold">Providers via ambiente</h2>
        </div>
        <div class="flex flex-wrap gap-2">
          <span v-for="provider in stats.general?.configured_env_providers ?? []" :key="provider" class="rounded-md bg-gray-100 px-2.5 py-1 text-xs font-medium dark:bg-slate-800">
            {{ provider }}
          </span>
          <span v-if="!(stats.general?.configured_env_providers ?? []).length" class="text-sm text-gray-500">Nenhum provider configurado por .env.</span>
        </div>
      </section>
    </section>
  </AppShell>
</template>
