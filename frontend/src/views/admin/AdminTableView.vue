<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import AppShell from '../../layouts/AppShell.vue'
import { http } from '../../api/http'

const route = useRoute()
const rows = ref<Record<string, unknown>[]>([])

async function load() {
  const resource = route.params.resource
  const { data } = await http.get(`/api/admin/${resource}`)
  rows.value = Array.isArray(data) ? data : []
}

onMounted(load)
watch(() => route.params.resource, load)
</script>

<template>
  <AppShell>
    <section class="p-6">
      <h1 class="mb-5 text-2xl font-semibold">{{ route.params.resource }}</h1>
      <div class="overflow-auto rounded-lg bg-white shadow-sm">
        <table class="w-full text-left text-sm">
          <tbody>
            <tr v-for="(row, index) in rows" :key="index" class="border-b border-gray-100">
              <td class="p-3"><pre class="whitespace-pre-wrap">{{ row }}</pre></td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </AppShell>
</template>

