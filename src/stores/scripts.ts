import { defineStore } from 'pinia'
import { appDataDir, join } from '@tauri-apps/api/path'
import { copyFile, mkdir, stat } from '@tauri-apps/plugin-fs'
import { loadEntries, saveEntries } from '../lib/persist'
import { detectLang } from '../lib/language'
import { dirOf } from '../lib/runner'
import { t } from '../lib/i18n'
import type { RunScheme, ScriptEntry } from '../types'

export interface AddResult {
  ok: boolean
  message?: string
  entry?: ScriptEntry
}

function makeDefaultScheme(command: string): RunScheme {
  return {
    id: crypto.randomUUID(),
    name: t('detail.defaultScheme'),
    command,
    cdToScriptDir: true,
  }
}

/** 托管脚本目录：应用数据目录下的 scripts/，具体路径由 Tauri 按平台解析 */
async function managedDir(): Promise<string> {
  const dir = await join(await appDataDir(), 'scripts')
  await mkdir(dir, { recursive: true })
  return dir
}

/** 托管文件名：时间戳 + 原扩展名（保证语言识别仍然有效） */
function stampFileName(path: string): string {
  const base = path.split(/[\\/]/).pop() ?? ''
  const dot = base.lastIndexOf('.')
  const ext = dot > 0 ? base.slice(dot) : ''
  return `${Date.now()}${ext}`
}

export const useScriptsStore = defineStore('scripts', {
  state: () => ({
    entries: [] as ScriptEntry[],
    loaded: false,
  }),
  getters: {
    /** MRU 排序：最近使用 → 使用次数 → 名称 */
    sorted(state): ScriptEntry[] {
      return [...state.entries].sort(
        (a, b) =>
          b.lastUsedAt - a.lastUsedAt || b.useCount - a.useCount || a.name.localeCompare(b.name),
      )
    },
  },
  actions: {
    byPath(path: string): ScriptEntry | undefined {
      return this.entries.find((e) => e.path === path)
    },
    async load() {
      this.entries = await loadEntries()
      this.loaded = true
    },
    async persist() {
      await saveEntries(this.entries)
    },
    async addByPath(path: string): Promise<AddResult> {
      try {
        const meta = await stat(path)
        if (!meta.isFile) return { ok: false, message: t('store.notFile') }
      } catch {
        return { ok: false, message: t('store.notFound') }
      }
      const existing = this.byPath(path)
      if (existing) {
        existing.lastUsedAt = Date.now()
        existing.useCount += 1
        await this.persist()
        return { ok: true, entry: existing }
      }
      const lang = detectLang(path)
      const scheme = makeDefaultScheme(lang.defaultCommand)
      const entry: ScriptEntry = {
        path,
        name: path.split(/[\\/]/).pop() ?? path,
        lang: lang.id,
        lastUsedAt: Date.now(),
        useCount: 1,
        schemes: [scheme],
        activeSchemeId: scheme.id,
      }
      this.entries.push(entry)
      await this.persist()
      return { ok: true, entry }
    },
    /**
     * 托管：把脚本复制到应用数据目录下的 scripts/，以时间戳命名。
     * 新条目保留原脚本名、原始路径与全部运行方案，默认在原目录执行。
     */
    async manage(path: string): Promise<AddResult> {
      const src = this.byPath(path)
      if (!src) return { ok: false, message: t('store.notFound') }
      try {
        const dir = await managedDir()
        const target = await join(dir, stampFileName(path))
        await copyFile(path, target)
        const entry: ScriptEntry = {
          path: target,
          name: src.name,
          lang: src.lang,
          lastUsedAt: Date.now(),
          useCount: 0,
          schemes: src.schemes.map((s) => ({ ...s })),
          activeSchemeId: src.activeSchemeId,
          managed: true,
          sourcePath: path,
          workDir: dirOf(path),
        }
        this.entries.push(entry)
        await this.persist()
        return { ok: true, entry }
      } catch (e) {
        return { ok: false, message: t('store.manageFailed', { err: String(e) }) }
      }
    },
    /** 托管脚本：修改执行目录（随时可改，立即持久化） */
    async setWorkDir(path: string, workDir: string) {
      const e = this.byPath(path)
      if (!e) return
      e.workDir = workDir
      await this.persist()
    },
    async remove(path: string) {
      const i = this.entries.findIndex((e) => e.path === path)
      if (i >= 0) {
        this.entries.splice(i, 1)
        await this.persist()
      }
    },
    async markUsed(path: string) {
      const e = this.byPath(path)
      if (!e) return
      e.lastUsedAt = Date.now()
      e.useCount += 1
      await this.persist()
    },
    /** 方案变更立即持久化（自动保存） */
    async saveSchemes(path: string, schemes: RunScheme[], activeSchemeId: string | null) {
      const e = this.byPath(path)
      if (!e) return
      e.schemes = schemes
      e.activeSchemeId = activeSchemeId
      await this.persist()
    },
  },
})
