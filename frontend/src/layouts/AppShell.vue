<script setup lang="ts">
import { BarChart3, Bot, KeyRound, Menu, MessageSquare, PanelLeftClose, Settings, UserRound, Wrench } from '@lucide/vue'
import { ref, watch } from 'vue'

defineProps<{
  fixed?: boolean
}>()

const storageKey = 'server-assistant-main-sidebar-collapsed'
const isCollapsed = ref(localStorage.getItem(storageKey) === 'true')

watch(isCollapsed, (value) => {
  localStorage.setItem(storageKey, String(value))
})
</script>

<template>
  <div
    class="grid bg-mist text-ink transition-[grid-template-columns] duration-200 dark:bg-slate-950 dark:text-slate-100"
    :class="[
      isCollapsed ? 'grid-cols-[64px_1fr]' : 'grid-cols-[280px_1fr]',
      fixed ? 'h-full min-h-0 overflow-hidden' : 'min-h-full'
    ]"
  >
    <aside class="border-r border-gray-200 bg-white dark:border-slate-800 dark:bg-slate-900" :class="fixed ? 'min-h-0 overflow-auto' : 'min-h-screen'">
      <div class="flex h-14 items-center gap-2 border-b border-gray-200 px-3 font-semibold dark:border-slate-800" :class="isCollapsed ? 'justify-center' : 'justify-between'">
        <div v-if="!isCollapsed" class="flex min-w-0 items-center gap-2">
          <Bot class="h-5 w-5 shrink-0 text-brand" />
          <span v-if="!isCollapsed" class="truncate">Server Assistant</span>
        </div>
        <button
          class="grid h-8 w-8 shrink-0 place-items-center rounded-md text-gray-500 hover:bg-gray-100 hover:text-brand dark:text-slate-300 dark:hover:bg-slate-800"
          :title="isCollapsed ? 'Abrir menu' : 'Engavetar menu'"
          @click="isCollapsed = !isCollapsed"
        >
          <Menu v-if="isCollapsed" class="h-4 w-4" />
          <PanelLeftClose v-else class="h-4 w-4" />
        </button>
      </div>
      <nav class="space-y-1 p-3 text-sm">
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="Chat" to="/"><MessageSquare class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">Chat</span></RouterLink>
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="Dashboard" to="/admin"><BarChart3 class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">Dashboard</span></RouterLink>
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="Providers" to="/admin/providers"><KeyRound class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">Providers</span></RouterLink>
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="Modelos" to="/admin/models"><Settings class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">Modelos</span></RouterLink>
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="MCP" to="/admin/mcp-tools"><Wrench class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">MCP</span></RouterLink>
        <RouterLink class="flex items-center gap-2 rounded-md px-3 py-2 hover:bg-gray-100 dark:hover:bg-slate-800" :class="isCollapsed ? 'justify-center' : ''" title="Perfil" to="/profile"><UserRound class="h-4 w-4 shrink-0" /><span v-if="!isCollapsed">Perfil</span></RouterLink>
      </nav>
    </aside>
    <main class="min-w-0" :class="fixed ? 'min-h-0 overflow-hidden' : ''">
      <slot />
    </main>
  </div>
</template>
