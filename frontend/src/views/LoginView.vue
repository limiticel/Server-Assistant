<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const email = ref('')
const password = ref('')
const auth = useAuthStore()
const router = useRouter()

async function submit() {
  await auth.login(email.value, password.value)
  await router.push('/')
}
</script>

<template>
  <main class="grid h-full place-items-center bg-mist text-ink">
    <form class="w-full max-w-sm rounded-lg bg-white p-6 shadow-sm" @submit.prevent="submit">
      <h1 class="mb-5 text-xl font-semibold">Login</h1>
      <input v-model="email" class="mb-3 w-full rounded-md border border-gray-300 px-3 py-2" placeholder="Email" />
      <input v-model="password" class="mb-4 w-full rounded-md border border-gray-300 px-3 py-2" placeholder="Senha" type="password" />
      <button class="w-full rounded-md bg-brand px-3 py-2 font-medium text-white">Entrar</button>
    </form>
  </main>
</template>
