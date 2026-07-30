import { defineStore } from 'pinia'
import { http } from '../api/http'

export const useAuthStore = defineStore('auth', {
  state: () => ({
    accessToken: localStorage.getItem('access_token'),
    refreshToken: localStorage.getItem('refresh_token'),
    role: localStorage.getItem('role') ?? 'user'
  }),
  actions: {
    async login(email: string, password: string) {
      const { data } = await http.post('/api/auth/login', { email, password })
      this.accessToken = data.access_token
      this.refreshToken = data.refresh_token
      this.role = data.role
      localStorage.setItem('access_token', data.access_token)
      localStorage.setItem('refresh_token', data.refresh_token)
      localStorage.setItem('role', data.role)
    },
    logout() {
      this.accessToken = null
      this.refreshToken = null
      localStorage.removeItem('access_token')
      localStorage.removeItem('refresh_token')
      localStorage.removeItem('role')
    }
  }
})
