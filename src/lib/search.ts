import { Command } from '@tauri-apps/plugin-shell'
import { readTextFile } from '@tauri-apps/plugin-fs'
import type { ScriptEntry } from '../types'
import { dirOf } from './runner'

/** 卡片展示的执行目录：托管脚本取可配置的执行目录，普通脚本取文件所在目录 */
function entryDir(entry: ScriptEntry): string {
  if (!entry.managed) return dirOf(entry.path)
  return entry.workDir || dirOf(entry.sourcePath ?? entry.path)
}

/** 文件名匹配（keyword 需已转为小写） */
export function matchName(entry: ScriptEntry, keyword: string): boolean {
  return entry.name.toLowerCase().includes(keyword)
}

/** 目录匹配（keyword 需已转为小写） */
export function matchDir(entry: ScriptEntry, keyword: string): boolean {
  return entryDir(entry).toLowerCase().includes(keyword)
}

type TextTool = 'rg' | 'grep' | null

let toolPromise: Promise<TextTool> | null = null

async function probe(tool: 'rg' | 'grep'): Promise<boolean> {
  try {
    const out = await Command.create(tool, ['--version']).execute()
    return out.code === 0
  } catch {
    return false
  }
}

/** 探测系统可用的文本搜索工具（只探测一次，优先 ripgrep） */
function detectTool(): Promise<TextTool> {
  if (!toolPromise) {
    toolPromise = (async () => {
      if (await probe('rg')) return 'rg'
      if (await probe('grep')) return 'grep'
      return null
    })()
  }
  return toolPromise
}

/** 前端兜底：逐个读取文件内容并做大小写不敏感的子串匹配 */
async function scanFiles(paths: string[], keyword: string, matched: Set<string>) {
  const batchSize = 16
  for (let i = 0; i < paths.length; i += batchSize) {
    await Promise.all(
      paths.slice(i, i + batchSize).map(async (path) => {
        try {
          if ((await readTextFile(path)).toLowerCase().includes(keyword)) matched.add(path)
        } catch {
          // 二进制文件或读取失败时跳过
        }
      }),
    )
  }
}

/**
 * 在给定文件中搜索内容，返回命中文件路径集合。
 * 优先使用系统自带的 ripgrep / grep；都不可用时回退为前端逐个读取文件。
 * 每次搜索即席执行，不建立索引。
 */
export async function searchContents(paths: string[], keyword: string): Promise<Set<string>> {
  const matched = new Set<string>()
  if (!paths.length || !keyword) return matched

  const tool = await detectTool()
  if (tool) {
    try {
      // -l 只输出命中文件名；-i 忽略大小写；-F 按字面量匹配；-- 隔离用户输入
      const args =
        tool === 'rg'
          ? ['-uu', '-l', '-i', '-F', '--', keyword, ...paths]
          : ['-l', '-i', '-F', '-s', '--', keyword, ...paths]
      const out = await Command.create(tool, args).execute()
      const known = new Set(paths)
      for (const line of out.stdout.split(/\r?\n/)) {
        const path = line.trim()
        if (path && known.has(path)) matched.add(path)
      }
      return matched
    } catch {
      // 工具执行异常时继续走前端兜底
    }
  }

  await scanFiles(paths, keyword, matched)
  return matched
}
