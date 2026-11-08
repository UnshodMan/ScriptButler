import { defineStore } from 'pinia'
import * as api from '../lib/api'
import { t } from '../lib/i18n'
import type { ScriptEntry } from '../types'

export interface AddResult {
  ok: boolean
  message?: string
  entry?: ScriptEntry
}

/** 后端数据的薄缓存：每步调用 Rust 命令并整体替换已排序的条目列表 */
export const useScriptsStore = defineStore('scripts', {
  state: () => ({
    entries: [] as ScriptEntry[],
    loaded: false,
  }),
  actions: {
    byPath(path: string): ScriptEntry | undefined {
      return this.entries.find((e) => e.path === path)
    },
    async load() {
      const res = await api.listScripts()
      this.entries = res.entries
      this.loaded = true
    },
    async addByPath(path: string): Promise<AddResult> {
      try {
        const res = await api.addScript(path, t('detail.defaultScheme'))
        this.entries = res.entries
        return { ok: true, entry: res.changed ?? undefined }
      } catch (e) {
        return { ok: false, message: api.errorText(e) }
      }
    },
    async manage(path: string): Promise<AddResult> {
      try {
        const res = await api.manageScript(path)
        this.entries = res.entries
        return { ok: true, entry: res.changed ?? undefined }
      } catch (e) {
        return { ok: false, message: api.errorText(e) }
      }
    },
    /** 托管脚本：修改执行目录（随时可改，立即持久化） */
    async setWorkDir(path: string, workDir: string) {
      this.entries = (await api.setWorkDir(path, workDir)).entries
    },
    async remove(path: string) {
      this.entries = (await api.removeScript(path)).entries
    },
    async markUsed(path: string) {
      this.entries = (await api.markUsed(path)).entries
    },
    /** 首次打开且无方案时，按语言生成默认方案 */
    async ensureDefaultScheme(path: string, name: string) {
      this.entries = (await api.ensureDefaultScheme(path, name)).entries
    },
    /** 方案变更立即持久化（自动保存） */
    async createScheme(path: string, name: string, command: string, cdToScriptDir: boolean) {
      this.entries = (await api.createScheme(path, name, command, cdToScriptDir)).entries
    },
    async updateScheme(
      path: string,
      id: string,
      name: string,
      command: string,
      cdToScriptDir: boolean,
    ) {
      this.entries = (await api.updateScheme(path, id, name, command, cdToScriptDir)).entries
    },
    async deleteScheme(path: string, id: string) {
      this.entries = (await api.deleteScheme(path, id)).entries
    },
    async setActiveScheme(path: string, id: string) {
      this.entries = (await api.setActiveScheme(path, id)).entries
    },
  },
})
