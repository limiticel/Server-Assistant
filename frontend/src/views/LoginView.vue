<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { AlertCircle, Bot, LockKeyhole, Mail } from '@lucide/vue'
import { useAuthStore } from '../stores/auth'

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const auth = useAuthStore()
const router = useRouter()
const route = useRoute()

async function submit() {
  error.value = ''
  loading.value = true

  try {
    await auth.login(email.value.trim(), password.value)
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/'
    await router.push(redirect)
  } catch {
    error.value = 'Email ou senha invalidos.'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <main class="grid min-h-full place-items-center bg-mist px-4 py-10 text-ink dark:bg-slate-950 dark:text-slate-100">
    <form
      class="w-full max-w-[420px] rounded-lg border border-gray-200 bg-white p-6 shadow-sm dark:border-slate-800 dark:bg-slate-900"
      @submit.prevent="submit"
    >
      <div class="mb-6 flex items-center gap-3">
        <div class="grid h-11 w-11 place-items-center rounded-lg bg-brand text-white">
          <Bot class="h-5 w-5" />
        </div>
        <div>
          <h1 class="text-lg font-semibold">Server Assistant</h1>
          <p class="text-sm text-gray-500 dark:text-slate-400">Entre para acessar o ambiente.</p>
        </div>
      </div>

      <label class="mb-2 block text-sm font-medium text-gray-700 dark:text-slate-300" for="email">Email</label>
      <div class="mb-4 flex items-center gap-2 rounded-md border border-gray-300 bg-white px-3 py-2 focus-within:border-brand dark:border-slate-700 dark:bg-slate-950">
        <Mail class="h-4 w-4 text-gray-400" />
        <input
          id="email"
          v-model="email"
          autocomplete="email"
          class="min-w-0 flex-1 bg-transparent text-sm outline-none"
          placeholder="seu@email.com"
          required
          type="email"
        />
      </div>

      <label class="mb-2 block text-sm font-medium text-gray-700 dark:text-slate-300" for="password">Senha</label>
      <div class="mb-4 flex items-center gap-2 rounded-md border border-gray-300 bg-white px-3 py-2 focus-within:border-brand dark:border-slate-700 dark:bg-slate-950">
        <LockKeyhole class="h-4 w-4 text-gray-400" />
        <input
          id="password"
          v-model="password"
          autocomplete="current-password"
          class="min-w-0 flex-1 bg-transparent text-sm outline-none"
          placeholder="Sua senha"
          required
          type="password"
        />
      </div>

      <div v-if="error" class="mb-4 flex items-center gap-2 rounded-md border border-red-200 bg-red-50 px-3 py-2 text-sm text-red-700 dark:border-red-900/60 dark:bg-red-950/30 dark:text-red-200">
        <AlertCircle class="h-4 w-4 shrink-0" />
        <span>{{ error }}</span>
      </div>

      <button
        class="grid h-10 w-full place-items-center rounded-md bg-brand px-3 text-sm font-semibold text-white transition hover:bg-brand-dark disabled:cursor-not-allowed disabled:opacity-70"
        :disabled="loading"
      >
        {{ loading ? 'Entrando...' : 'Entrar' }}
      </button>
    </form>
  </main>
</template>
