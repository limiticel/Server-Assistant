<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { Check, CornerDownLeft, Menu, PanelLeftClose, Pencil, Plus, Send, Square, Terminal, Trash2, X } from '@lucide/vue'
import ConfirmModal from '../../components/ConfirmModal.vue'
import MarkdownMessage from '../../components/MarkdownMessage.vue'
import AppShell from '../../layouts/AppShell.vue'
import { useChatStore } from '../../stores/chat'

const chat = useChatStore()
const input = ref('')
const messagesContainer = ref<HTMLElement | null>(null)
const conversationToDelete = ref('')
const editingConversationId = ref('')
const editingTitle = ref('')
const selectedModels = computed(() => chat.modelsForProvider)
const conversationSidebarKey = 'server-assistant-conversation-sidebar-collapsed'
const isConversationSidebarCollapsed = ref(localStorage.getItem(conversationSidebarKey) === 'true')
const sendWithEnterKey = 'server-assistant-send-with-enter'
const sendWithEnter = ref(localStorage.getItem(sendWithEnterKey) === 'true')

watch(isConversationSidebarCollapsed, (value) => {
  localStorage.setItem(conversationSidebarKey, String(value))
})

watch(sendWithEnter, (value) => {
  localStorage.setItem(sendWithEnterKey, String(value))
})

onMounted(async () => {
  await Promise.all([chat.loadConversations(), chat.loadAiOptions()])
  scrollMessagesToBottom('auto')
})

watch(
  () => [
    chat.activeConversationId,
    chat.messages.length,
    chat.messages.at(-1)?.content ?? '',
    chat.messages.at(-1)?.steps?.length ?? 0
  ],
  () => scrollMessagesToBottom('auto')
)

let pendingScrollFrame = 0

async function scrollMessagesToBottom(behavior: ScrollBehavior = 'auto') {
  await nextTick()
  if (pendingScrollFrame) window.cancelAnimationFrame(pendingScrollFrame)

  pendingScrollFrame = window.requestAnimationFrame(() => {
    const element = messagesContainer.value
    if (!element) return
    element.scrollTo({
      top: element.scrollHeight,
      behavior
    })
    pendingScrollFrame = 0
  })
}

async function send() {
  if (chat.activeConversationSending) return
  const content = input.value.trim()
  if (!content) return
  input.value = ''
  await chat.send(content)
}

async function handleComposerKeydown(event: KeyboardEvent) {
  if (!sendWithEnter.value) return
  if (event.key !== 'Enter' || event.shiftKey || event.isComposing) return
  event.preventDefault()
  await send()
}

function requestDeleteConversation(conversationId: string) {
  conversationToDelete.value = conversationId
}

async function confirmDeleteConversation() {
  if (!conversationToDelete.value) return
  await chat.deleteConversation(conversationToDelete.value)
  conversationToDelete.value = ''
}

function startRenameConversation(conversationId: string, currentTitle: string) {
  editingConversationId.value = conversationId
  editingTitle.value = currentTitle
}

function cancelRenameConversation() {
  editingConversationId.value = ''
  editingTitle.value = ''
}

async function saveRenameConversation() {
  if (!editingConversationId.value) return
  await chat.renameConversation(editingConversationId.value, editingTitle.value)
  cancelRenameConversation()
}
</script>

<template>
  <AppShell fixed>
    <div
      class="grid h-full min-h-0 overflow-hidden transition-[grid-template-columns] duration-200"
      :class="isConversationSidebarCollapsed ? 'grid-cols-[56px_1fr]' : 'grid-cols-[260px_1fr]'"
    >
      <section
        class="min-h-0 overflow-y-auto overflow-x-hidden border-r border-gray-200 bg-white"
        :class="isConversationSidebarCollapsed ? 'px-2 py-3' : 'p-3'"
      >
        <div v-if="!isConversationSidebarCollapsed" class="mb-3 flex items-center justify-center gap-2">
          <button
            class="grid h-9 shrink-0 flex-1 place-items-center rounded-md bg-brand px-3 text-sm font-medium text-white"
            @click="chat.newConversation"
          >
            Novo Chat
          </button>
          <button
            class="grid h-9 w-9 shrink-0 place-items-center rounded-md border border-gray-300 text-gray-500 hover:bg-gray-100 hover:text-brand dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
            title="Engavetar conversas"
            @click="isConversationSidebarCollapsed = true"
          >
            <PanelLeftClose class="h-4 w-4" />
          </button>
        </div>
        <div v-else class="mb-3 flex justify-center">
          <button
            class="grid h-9 w-9 shrink-0 place-items-center rounded-md border border-gray-300 text-gray-500 hover:bg-gray-100 hover:text-brand dark:border-slate-700 dark:text-slate-300 dark:hover:bg-slate-800"
            title="Abrir conversas"
            @click="isConversationSidebarCollapsed = false"
          >
            <Menu class="h-4 w-4" />
          </button>
        </div>
        <div v-if="!isConversationSidebarCollapsed" class="space-y-1">
          <button
            v-for="item in chat.conversations"
            :key="item.id"
            class="group flex w-full items-center justify-between gap-2 rounded-md px-3 py-2 text-left text-sm hover:bg-gray-100"
            :class="chat.activeConversationId === item.id ? 'bg-gray-100 dark:bg-slate-800' : ''"
            @click="chat.selectConversation(item.id)"
          >
            <template v-if="editingConversationId === item.id">
              <input
                v-model="editingTitle"
                class="min-w-0 flex-1 rounded-md border border-gray-300 px-2 py-1 text-sm"
                autofocus
                @click.stop
                @keydown.enter.prevent.stop="saveRenameConversation"
                @keydown.esc.prevent.stop="cancelRenameConversation"
              />
              <span class="flex shrink-0 gap-1">
                <span class="grid h-7 w-7 place-items-center rounded-md text-brand hover:bg-gray-100 dark:hover:bg-slate-700" title="Salvar" @click.stop="saveRenameConversation">
                  <Check class="h-4 w-4" />
                </span>
                <span class="grid h-7 w-7 place-items-center rounded-md text-gray-400 hover:bg-gray-100 dark:hover:bg-slate-700" title="Cancelar" @click.stop="cancelRenameConversation">
                  <X class="h-4 w-4" />
                </span>
              </span>
            </template>
            <template v-else>
              <span class="flex min-w-0 flex-1 items-center gap-2">
                <span
                  v-if="chat.isConversationSending(item.id)"
                  class="relative h-2.5 w-2.5 shrink-0"
                  title="IA gerando resposta"
                >
                  <span class="absolute inset-0 rounded-full bg-emerald-400 opacity-75 animate-ping" />
                  <span class="absolute inset-0 rounded-full bg-emerald-400" />
                </span>
                <span class="min-w-0 flex-1 truncate">{{ item.title }}</span>
              </span>
              <span class="flex shrink-0 opacity-0 transition group-hover:opacity-100">
                <span
                  class="grid h-7 w-7 place-items-center rounded-md text-gray-400 hover:bg-gray-100 hover:text-brand dark:hover:bg-slate-700"
                  title="Renomear conversa"
                  @click.stop="startRenameConversation(item.id, item.title)"
                >
                  <Pencil class="h-4 w-4" />
                </span>
                <span
                  class="grid h-7 w-7 place-items-center rounded-md text-gray-400 hover:bg-red-50 hover:text-red-600 dark:hover:bg-red-950"
                  title="Excluir conversa"
                  @click.stop="requestDeleteConversation(item.id)"
                >
                  <Trash2 class="h-4 w-4" />
                </span>
              </span>
            </template>
          </button>
        </div>
        <div v-else class="space-y-2">
          <button
            v-for="item in chat.conversations"
            :key="item.id"
            class="grid h-8 w-8 place-items-center rounded-md text-xs font-semibold hover:bg-gray-100 dark:hover:bg-slate-800"
            :class="chat.activeConversationId === item.id ? 'bg-gray-100 text-brand dark:bg-slate-800' : 'text-gray-500 dark:text-slate-300'"
            :title="item.title"
            @click="chat.selectConversation(item.id)"
          >
            <span class="relative grid h-full w-full place-items-center">
              <span
                v-if="chat.isConversationSending(item.id)"
                class="absolute right-0 top-0 h-2 w-2 rounded-full bg-emerald-400"
              >
                <span class="absolute inset-0 rounded-full bg-emerald-400 animate-ping" />
              </span>
              {{ item.title.trim().charAt(0).toUpperCase() || 'C' }}
            </span>
          </button>
        </div>
      </section>

      <section class="flex h-full min-h-0 flex-col overflow-hidden">
        <header class="flex h-14 shrink-0 items-center justify-between border-b border-gray-200 bg-white px-4">
          <div class="flex gap-2">
            <button
              v-if="isConversationSidebarCollapsed"
              class="inline-flex items-center gap-2 rounded-md bg-brand px-3 py-1.5 text-sm font-medium text-white"
              title="Novo Chat"
              @click="chat.newConversation"
            >
              <Plus class="h-4 w-4" />
              Novo Chat
            </button>
            <select :value="chat.provider" class="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm" @change="chat.selectProvider(($event.target as HTMLSelectElement).value)">
              <option v-if="!chat.providers.length" value="ollama">ollama</option>
              <option v-for="provider in chat.providers" :key="provider.id" :value="provider.id">
                {{ provider.name }}
              </option>
            </select>
            <select
              v-if="selectedModels.length"
              :value="chat.model"
              class="rounded-md border border-gray-300 bg-white px-2 py-1 text-sm"
              @change="chat.selectModel(($event.target as HTMLSelectElement).value)"
            >
              <option v-for="model in selectedModels" :key="model.id" :value="model.name">{{ model.name }}</option>
            </select>
            <input
              v-else
              :value="chat.model"
              class="rounded-md border border-gray-300 px-2 py-1 text-sm"
              placeholder="modelo"
              @input="chat.selectModel(($event.target as HTMLInputElement).value)"
            />
          </div>
        </header>

        <div ref="messagesContainer" class="min-h-0 flex-1 space-y-4 overflow-auto p-6">
          <article v-for="(message, index) in chat.messages" :key="index" class="mx-auto max-w-3xl">
            <div
              :class="message.role === 'user' ? 'ml-auto bg-brand text-white' : 'mr-auto bg-white text-ink'"
              class="w-fit max-w-[80%] overflow-hidden rounded-lg px-4 py-3 shadow-sm"
            >
              <pre v-if="message.role === 'user'" class="whitespace-pre-wrap font-sans text-sm leading-6">{{ message.content }}</pre>
              <template v-else>
                <div v-if="message.steps?.length" class="mb-3 space-y-2">
                  <details
                    v-for="(step, stepIndex) in message.steps"
                    :key="`${step.type}-${stepIndex}`"
                    class="rounded-md border border-gray-200 bg-gray-50 text-xs dark:border-slate-700 dark:bg-slate-900"
                    :open="step.type !== 'status'"
                  >
                    <summary class="flex cursor-pointer list-none items-center gap-2 px-3 py-2 font-medium text-gray-600 dark:text-slate-300">
                      <Terminal v-if="step.type !== 'status'" class="h-3.5 w-3.5 text-brand" />
                      <span v-else class="h-2 w-2 rounded-full bg-brand" />
                      {{ step.title }}
                    </summary>
                    <pre v-if="step.detail" class="max-h-48 overflow-auto border-t border-gray-200 p-3 font-mono text-[11px] leading-5 text-gray-600 dark:border-slate-700 dark:text-slate-300">{{ step.detail }}</pre>
                  </details>
                </div>
                <MarkdownMessage v-if="message.content" :content="message.content" />
                <div v-else class="h-5 w-12 animate-pulse rounded-full bg-gray-100 dark:bg-slate-800" />
              </template>
            </div>
          </article>
        </div>

        <form class="shrink-0 border-t border-gray-200 bg-white p-4" @submit.prevent="send">
          <div class="mx-auto flex max-w-3xl gap-2">
            <div class="flex flex-1 items-stretch rounded-md border border-gray-300 bg-white focus-within:border-brand focus-within:ring-1 focus-within:ring-brand dark:border-slate-600 dark:bg-slate-950">
              <textarea
                v-model="input"
                class="min-h-12 flex-1 resize-none border-0 bg-transparent p-3 text-sm outline-none"
                placeholder="Mensagem"
                @keydown="handleComposerKeydown"
              />
              <button
                class="m-2 grid h-8 w-8 shrink-0 place-items-center rounded-md border text-gray-500 transition hover:text-brand"
                :class="sendWithEnter ? 'border-brand bg-teal-50 text-brand dark:bg-teal-950' : 'border-gray-300 dark:border-slate-700 dark:text-slate-300'"
                type="button"
                :title="sendWithEnter ? 'Enter envia. Shift+Enter quebra linha.' : 'Enter quebra linha. Clique para enviar com Enter.'"
                @click="sendWithEnter = !sendWithEnter"
              >
                <CornerDownLeft class="h-4 w-4" />
              </button>
            </div>
            <button
              v-if="chat.activeConversationSending"
              class="grid h-12 w-12 place-items-center rounded-md bg-red-600 text-white transition hover:bg-red-500"
              type="button"
              title="Interromper resposta"
              @click="chat.cancelResponse()"
            >
              <Square class="h-4 w-4 fill-current" />
            </button>
            <button v-else class="grid h-12 w-12 place-items-center rounded-md bg-brand text-white" type="submit" title="Enviar">
              <Send class="h-5 w-5" />
            </button>
          </div>
        </form>
      </section>
    </div>
    <ConfirmModal
      :open="Boolean(conversationToDelete)"
      title="Excluir conversa"
      message="Esta conversa e suas mensagens serao removidas."
      confirm-label="Excluir"
      tone="danger"
      @cancel="conversationToDelete = ''"
      @confirm="confirmDeleteConversation"
    />
  </AppShell>
</template>
