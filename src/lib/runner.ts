import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as api from './api'
import type { RunEvent, RunPayload } from '../types'

type Handler = (e: RunEvent) => void

const handlers = new Map<string, Set<Handler>>()

function emit(runId: string, e: RunEvent) {
  handlers.get(runId)?.forEach((h) => h(e))
}

/** 订阅某次运行的输出事件，返回取消订阅函数 */
export function subscribe(runId: string, handler: Handler): () => void {
  let set = handlers.get(runId)
  if (!set) {
    set = new Set()
    handlers.set(runId, set)
  }
  set.add(handler)
  return () => {
    set!.delete(handler)
    if (set!.size === 0) handlers.delete(runId)
  }
}

let listener: Promise<UnlistenFn> | null = null

/** 全局只注册一次运行事件监听 */
export function ensureRunListener(): Promise<UnlistenFn> {
  if (!listener) {
    listener = listen<RunPayload>('run://event', (e) => {
      const { runId, ...event } = e.payload
      emit(runId, event)
    })
  }
  return listener
}

export async function startRun(path: string, schemeId: string, args?: string): Promise<string> {
  await ensureRunListener()
  return api.startRun(path, schemeId, args)
}

export async function stopRun(runId: string): Promise<void> {
  await api.stopRun(runId)
}
