<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { AlertTriangle } from '@lucide/vue'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

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
    by_provider: Array<{
      provider: string
      provider_type: string
      prompt_tokens: number
      completion_tokens: number
      total_tokens: number
      estimated_cost: string
    }>
  }
}

const stats = ref<DashboardStats>({})
const generalCards = computed(() => [
  { label: 'Usuários', value: stats.value.general?.users ?? 0 },
  { label: 'Conversas', value: stats.value.general?.conversations ?? 0 },
  { label: 'Mensagens', value: stats.value.general?.messages ?? 0 }
])

const billableCards = computed(() => [
  { label: 'Prompt tokens', value: stats.value.billable_usage?.prompt_tokens ?? 0 },
  { label: 'Completion tokens', value: stats.value.billable_usage?.completion_tokens ?? 0 },
  { label: 'Total tokens', value: stats.value.billable_usage?.total_tokens ?? 0 },
  { label: 'Custo estimado', value: `$${stats.value.billable_usage?.estimated_cost ?? '0'}` }
])

onMounted(async () => {
  const { data } = await http.get('/api/admin/dashboard')
  stats.value = data
})
</script>

<template>
  <AppShell>
    <section class="p-6">
      <h1 class="mb-5 text-2xl font-semibold">Dashboard</h1>

      <div class="mb-5 flex items-start gap-3 rounded-lg border border-amber-200 bg-amber-50 p-4 text-amber-900 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-100">
        <AlertTriangle class="mt-0.5 h-5 w-5 shrink-0" />
        <p class="text-sm leading-6">
          As métricas de tokens e custo estimado contabilizam somente APIs OpenAI e Claude/Anthropic.
          Providers locais ou compatíveis, como Ollama e outros, aparecem no uso geral, mas ainda não entram no cálculo financeiro.
        </p>
      </div>

      <div class="mb-6">
        <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">Uso geral</h2>
        <div class="grid grid-cols-3 gap-4">
          <div v-for="card in generalCards" :key="card.label" class="rounded-lg bg-white p-4 shadow-sm">
            <div class="text-sm text-gray-500">{{ card.label }}</div>
            <div class="mt-2 text-2xl font-semibold">{{ card.value }}</div>
          </div>
        </div>
      </div>

      <div class="mb-6">
        <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">Uso contabilizado</h2>
        <div class="grid grid-cols-4 gap-4">
          <div v-for="card in billableCards" :key="card.label" class="rounded-lg bg-white p-4 shadow-sm">
            <div class="text-sm text-gray-500">{{ card.label }}</div>
            <div class="mt-2 text-2xl font-semibold">{{ card.value }}</div>
          </div>
        </div>
      </div>

      <div class="overflow-auto rounded-lg bg-white shadow-sm">
        <table class="w-full text-left text-sm">
          <thead class="border-b border-gray-100 text-gray-500">
            <tr>
              <th class="p-3">Provider</th>
              <th class="p-3">Tipo</th>
              <th class="p-3">Prompt</th>
              <th class="p-3">Completion</th>
              <th class="p-3">Total</th>
              <th class="p-3">Custo</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in stats.billable_usage?.by_provider ?? []" :key="`${row.provider}-${row.provider_type}`" class="border-b border-gray-100">
              <td class="p-3 font-medium">{{ row.provider }}</td>
              <td class="p-3">{{ row.provider_type }}</td>
              <td class="p-3">{{ row.prompt_tokens }}</td>
              <td class="p-3">{{ row.completion_tokens }}</td>
              <td class="p-3">{{ row.total_tokens }}</td>
              <td class="p-3">${{ row.estimated_cost }}</td>
            </tr>
            <tr v-if="!(stats.billable_usage?.by_provider ?? []).length">
              <td class="p-3 text-gray-500" colspan="6">Nenhum uso contabilizado de OpenAI ou Claude ainda.</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div class="mt-6 rounded-lg bg-white p-4 shadow-sm">
        <div class="text-sm text-gray-500">Providers via ambiente</div>
        <div class="mt-2 flex flex-wrap gap-2">
          <span v-for="provider in stats.general?.configured_env_providers ?? []" :key="provider" class="rounded-md bg-gray-100 px-2 py-1 text-xs">
            {{ provider }}
          </span>
          <span v-if="!(stats.general?.configured_env_providers ?? []).length" class="text-sm text-gray-500">Nenhum provider configurado por .env.</span>
        </div>
      </div>
    </section>
  </AppShell>
</template>
