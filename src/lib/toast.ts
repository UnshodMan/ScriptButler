import { reactive } from 'vue'

export interface ToastItem {
  id: number
  text: string
  kind: 'info' | 'error'
}

export const toasts = reactive<ToastItem[]>([])

let seq = 0

export function toast(text: string, kind: 'info' | 'error' = 'info') {
  const id = ++seq
  toasts.push({ id, text, kind })
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id)
    if (i >= 0) toasts.splice(i, 1)
  }, 3000)
}
