import { createRouter, createWebHistory } from 'vue-router'
import ChatView from '../views/chat/ChatView.vue'
import LoginView from '../views/LoginView.vue'
import RegisterView from '../views/RegisterView.vue'
import DashboardView from '../views/admin/DashboardView.vue'
import AdminTableView from '../views/admin/AdminTableView.vue'
import ProvidersView from '../views/admin/ProvidersView.vue'
import McpToolsView from '../views/admin/McpToolsView.vue'
import ModelsView from '../views/admin/ModelsView.vue'
import ProfileView from '../views/ProfileView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: ChatView, meta: { requiresAuth: true } },
    { path: '/login', component: LoginView, meta: { guestOnly: true } },
    { path: '/register', component: RegisterView, meta: { guestOnly: true } },
    { path: '/profile', component: ProfileView, meta: { requiresAuth: true } },
    { path: '/admin', component: DashboardView, meta: { requiresAuth: true } },
    { path: '/admin/providers', component: ProvidersView, meta: { requiresAuth: true } },
    { path: '/admin/models', component: ModelsView, meta: { requiresAuth: true } },
    { path: '/admin/mcp-tools', component: McpToolsView, meta: { requiresAuth: true } },
    { path: '/admin/:resource', component: AdminTableView, meta: { requiresAuth: true } }
  ]
})

router.beforeEach((to) => {
  const hasToken = Boolean(localStorage.getItem('access_token'))

  if (to.meta.requiresAuth && !hasToken) {
    return { path: '/login', query: { redirect: to.fullPath } }
  }

  if (to.meta.guestOnly && hasToken) {
    return '/'
  }

  return true
})

export default router
