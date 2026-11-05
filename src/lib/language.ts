import type { Extension } from '@codemirror/state'
import { javascript } from '@codemirror/lang-javascript'
import { python } from '@codemirror/lang-python'
import { json } from '@codemirror/lang-json'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { markdown } from '@codemirror/lang-markdown'
import { yaml } from '@codemirror/lang-yaml'
import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'
import { t } from './i18n'

export interface LangInfo {
  id: string
  label: string
  /** 首次添加脚本时自动生成的默认运行方案命令模板 */
  defaultCommand: string
}

const PLAIN: LangInfo = { id: 'plain', label: 'plain', defaultCommand: '{file}' }

const EXT_MAP: Record<string, LangInfo> = {
  py: { id: 'python', label: 'Python', defaultCommand: 'python3 {file}' },
  js: { id: 'javascript', label: 'JavaScript', defaultCommand: 'node {file}' },
  mjs: { id: 'javascript', label: 'JavaScript', defaultCommand: 'node {file}' },
  cjs: { id: 'javascript', label: 'JavaScript', defaultCommand: 'node {file}' },
  ts: { id: 'typescript', label: 'TypeScript', defaultCommand: 'npx tsx {file}' },
  mts: { id: 'typescript', label: 'TypeScript', defaultCommand: 'npx tsx {file}' },
  tsx: { id: 'typescript', label: 'TypeScript', defaultCommand: 'npx tsx {file}' },
  sh: { id: 'shell', label: 'Shell', defaultCommand: 'bash {file}' },
  bash: { id: 'shell', label: 'Shell', defaultCommand: 'bash {file}' },
  zsh: { id: 'shell', label: 'Shell', defaultCommand: 'bash {file}' },
  json: { id: 'json', label: 'JSON', defaultCommand: 'cat {file}' },
  yaml: { id: 'yaml', label: 'YAML', defaultCommand: 'cat {file}' },
  yml: { id: 'yaml', label: 'YAML', defaultCommand: 'cat {file}' },
  md: { id: 'markdown', label: 'Markdown', defaultCommand: 'cat {file}' },
  html: { id: 'html', label: 'HTML', defaultCommand: 'open {file}' },
  htm: { id: 'html', label: 'HTML', defaultCommand: 'open {file}' },
  css: { id: 'css', label: 'CSS', defaultCommand: 'open {file}' },
}

export function detectLang(path: string): LangInfo {
  const dot = path.lastIndexOf('.')
  const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  if (dot <= slash || dot === path.length - 1) return PLAIN
  const ext = path.slice(dot + 1).toLowerCase()
  return EXT_MAP[ext] ?? PLAIN
}

export function languageLabel(langId: string): string {
  if (langId === 'plain') return t('lang.plain')
  return Object.values(EXT_MAP).find((l) => l.id === langId)?.label ?? t('lang.plain')
}

/** 按语言 id 返回 CodeMirror 语言扩展（高亮 + 关键词补全数据） */
export function languageExtension(langId: string): Extension | null {
  switch (langId) {
    case 'python':
      return python()
    case 'javascript':
      return javascript()
    case 'typescript':
      return javascript({ typescript: true })
    case 'json':
      return json()
    case 'html':
      return html()
    case 'css':
      return css()
    case 'markdown':
      return markdown()
    case 'yaml':
      return yaml()
    case 'shell':
      return StreamLanguage.define(shell)
    default:
      return null
  }
}
