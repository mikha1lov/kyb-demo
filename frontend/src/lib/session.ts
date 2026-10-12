const key = 'kyb-proto:application'

export function currentApplication(): string | null {
  if (typeof window === 'undefined') return null
  try {
    return window.localStorage.getItem(key)
  } catch {
    return null
  }
}

export function rememberApplication(id: string) {
  try {
    window.localStorage.setItem(key, id)
  } catch {
    return
  }
}

export function forgetApplication() {
  try {
    window.localStorage.removeItem(key)
  } catch {
    return
  }
}
