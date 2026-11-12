import { invoke } from '@tauri-apps/api/core'
import { t } from './i18n'
import type { ApiError, ScriptList, SearchGroups } from '../types'

/** Rust 后端能力的类型化封装，前端不直接触碰文件系统与子进程 */

export const listScripts = () => invoke<ScriptList>('list_scripts')

export const addScript = (path: string, defaultSchemeName: string) =>
  invoke<ScriptList>('add_script', { path, defaultSchemeName })

export const manageScript = (path: string) => invoke<ScriptList>('manage_script', { path })

export const removeScript = (path: string) => invoke<ScriptList>('remove_script', { path })

export const setWorkDir = (path: string, workDir: string) =>
  invoke<ScriptList>('set_work_dir', { path, workDir })

export const markUsed = (path: string) => invoke<ScriptList>('mark_used', { path })

export const ensureDefaultScheme = (path: string, name: string) =>
  invoke<ScriptList>('ensure_default_scheme', { path, name })

export const createScheme = (
  path: string,
  name: string,
  command: string,
  cdToScriptDir: boolean,
) => invoke<ScriptList>('create_scheme', { path, name, command, cdToScriptDir })

export const updateScheme = (
  path: string,
  id: string,
  name: string,
  command: string,
  cdToScriptDir: boolean,
) => invoke<ScriptList>('update_scheme', { path, id, name, command, cdToScriptDir })

export const deleteScheme = (path: string, id: string) =>
  invoke<ScriptList>('delete_scheme', { path, id })

export const setActiveScheme = (path: string, id: string) =>
  invoke<ScriptList>('set_active_scheme', { path, id })

export const readScript = (path: string) => invoke<string>('read_script', { path })

export const writeScript = (path: string, content: string) =>
  invoke<void>('write_script', { path, content })

export const startRun = (path: string, schemeId: string, args?: string) =>
  invoke<string>('start_run', { path, schemeId, args: args?.trim() ? args : null })

export const stopRun = (runId: string) => invoke<void>('stop_run', { runId })

export const searchScripts = (query: string) =>
  invoke<SearchGroups>('search_scripts', { query })

export const searchContents = (paths: string[], keyword: string) =>
  invoke<string[]>('search_contents', { paths, keyword })

/** 把 Rust 错误码映射为当前语言的提示文案 */
export function errorText(e: unknown): string {
  const err = e as Partial<ApiError> | null
  switch (err?.code) {
    case 'not_found':
      return t('store.notFound')
    case 'not_file':
      return t('store.notFile')
    case 'manage_failed':
      return t('store.manageFailed', { err: err.detail ?? '' })
    case 'read_failed':
      return t('detail.readFailed', { err: err.detail ?? '' })
    case 'write_failed':
      return t('detail.saveFailed', { err: err.detail ?? '' })
    case 'spawn_failed':
      return t('detail.startFailed', { err: err.detail ?? '' })
    default:
      return String(e)
  }
}
