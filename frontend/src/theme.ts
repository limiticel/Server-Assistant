export type ThemeMode = 'light' | 'dark'

const storageKey = 'server_assistant_theme'

export function getSavedTheme(): ThemeMode {
  const saved = localStorage.getItem(storageKey)
  return saved === 'dark' ? 'dark' : 'light'
}

export function applyTheme(theme: ThemeMode) {
  document.documentElement.classList.toggle('dark', theme === 'dark')
  localStorage.setItem(storageKey, theme)
}

export function initializeTheme() {
  applyTheme(getSavedTheme())
}
