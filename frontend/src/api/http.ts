import axios from 'axios'

export const http = axios.create({
  baseURL: import.meta.env.VITE_API_URL ?? '',
  headers: {
    'Content-Type': 'application/json'
  }
})

http.interceptors.request.use((config) => {
  const token = localStorage.getItem('access_token')
  if (token) config.headers.Authorization = `Bearer ${token}`
  return config
})

let refreshRequest: Promise<string | null> | null = null

http.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config
    const refreshToken = localStorage.getItem('refresh_token')

    if (
      error.response?.status === 401 &&
      refreshToken &&
      !originalRequest?._retry &&
      !String(originalRequest?.url ?? '').includes('/api/auth/login') &&
      !String(originalRequest?.url ?? '').includes('/api/auth/refresh')
    ) {
      originalRequest._retry = true
      refreshRequest ??= refreshAccessToken(refreshToken).finally(() => {
        refreshRequest = null
      })

      const accessToken = await refreshRequest
      if (accessToken) {
        originalRequest.headers.Authorization = `Bearer ${accessToken}`
        return http(originalRequest)
      }
    }

    if (error.response?.status === 401 && window.location.pathname !== '/login') {
      clearAuth()
      window.location.assign(`/login?redirect=${encodeURIComponent(window.location.pathname)}`)
    }

    return Promise.reject(error)
  }
)

async function refreshAccessToken(refreshToken: string) {
  try {
    const { data } = await http.post('/api/auth/refresh', {
      refresh_token: refreshToken
    })
    localStorage.setItem('access_token', data.access_token)
    localStorage.setItem('refresh_token', data.refresh_token)
    localStorage.setItem('role', data.role)
    return data.access_token as string
  } catch {
    clearAuth()
    return null
  }
}

function clearAuth() {
  localStorage.removeItem('access_token')
  localStorage.removeItem('refresh_token')
  localStorage.removeItem('role')
}
