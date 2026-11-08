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

/** 语言 id → 展示名（语言识别与默认命令由 Rust 负责） */
const LABELS: Record<string, string> = {
  python: 'Python',
  javascript: 'JavaScript',
  typescript: 'TypeScript',
  shell: 'Shell',
  json: 'JSON',
  yaml: 'YAML',
  markdown: 'Markdown',
  html: 'HTML',
  css: 'CSS',
}

export function languageLabel(langId: string): string {
  if (langId === 'plain') return t('lang.plain')
  return LABELS[langId] ?? t('lang.plain')
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
