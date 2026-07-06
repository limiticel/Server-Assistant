<script setup lang="ts">
defineProps<{
  open: boolean
  title: string
  message: string
  confirmLabel?: string
  cancelLabel?: string
  tone?: 'danger' | 'default'
}>()

defineEmits<{
  cancel: []
  confirm: []
}>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="fixed inset-0 z-50 grid place-items-center bg-black/45 p-4" @click.self="$emit('cancel')">
      <section class="w-full max-w-md rounded-lg border border-gray-200 bg-white p-5 text-ink shadow-xl dark:border-slate-700 dark:bg-slate-900 dark:text-slate-100">
        <h2 class="text-lg font-semibold">{{ title }}</h2>
        <p class="mt-2 text-sm leading-6 text-gray-600 dark:text-slate-300">{{ message }}</p>
        <div class="mt-5 flex justify-end gap-2">
          <button class="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium hover:bg-gray-100 dark:border-slate-600 dark:hover:bg-slate-800" @click="$emit('cancel')">
            {{ cancelLabel ?? 'Cancelar' }}
          </button>
          <button
            class="rounded-md px-4 py-2 text-sm font-medium text-white"
            :class="tone === 'danger' ? 'bg-red-600 hover:bg-red-700' : 'bg-brand hover:bg-teal-700'"
            @click="$emit('confirm')"
          >
            {{ confirmLabel ?? 'Confirmar' }}
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
