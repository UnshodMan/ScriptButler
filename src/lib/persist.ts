import { LazyStore } from '@tauri-apps/plugin-store'
import type { ScriptEntry } from '../types'

const store = new LazyStore('scripts.json')
const KEY = 'entries'

export async function loadEntries(): Promise<ScriptEntry[]> {
  const entries = (await store.get<ScriptEntry[]>(KEY)) ?? []
  // 旧数据没有 cdToScriptDir 字段，补为默认值（开启）
  for (const e of entries) {
    for (const s of e.schemes) {
      if (typeof s.cdToScriptDir !== 'boolean') s.cdToScriptDir = true
    }
  }
  return entries
}

export async function saveEntries(entries: ScriptEntry[]): Promise<void> {
  await store.set(KEY, entries)
  await store.save()
}
