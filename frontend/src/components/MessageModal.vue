<script setup lang="ts">
defineProps<{
  open: boolean
  title: string
  message: string
  actionLabel?: string
  tone?: 'danger' | 'default'
}>()

defineEmits<{
  close: []
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 grid place-items-center bg-black/45 p-4" @click.self="$emit('close')">
      <section class="w-full max-w-md rounded-lg border border-gray-200 bg-white p-5 text-ink shadow-xl dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100">
        <div class="flex items-start gap-3">
          <div
            class="mt-1 h-2.5 w-2.5 shrink-0 rounded-full"
            :class="tone === 'danger' ? 'bg-red-500' : 'bg-brand'"
          />
          <div class="min-w-0">
            <h2 class="text-base font-semibold">{{ title }}</h2>
            <p class="mt-2 whitespace-pre-wrap break-words text-sm leading-6 text-gray-600 dark:text-slate-300">{{ message }}</p>
          </div>
        </div>
        <div class="mt-5 flex justify-end">
          <button
            class="rounded-md px-4 py-2 text-sm font-medium text-white"
            :class="tone === 'danger' ? 'bg-red-600 hover:bg-red-700' : 'bg-brand hover:bg-teal-700'"
            @click="$emit('close')"
          >
            {{ actionLabel ?? 'Fechar' }}
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
