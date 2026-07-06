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

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: ChatView },
    { path: '/login', component: LoginView },
    { path: '/register', component: RegisterView },
    { path: '/profile', component: ProfileView },
    { path: '/admin', component: DashboardView },
    { path: '/admin/providers', component: ProvidersView },
    { path: '/admin/models', component: ModelsView },
    { path: '/admin/mcp-tools', component: McpToolsView },
    { path: '/admin/:resource', component: AdminTableView }
  ]
})
