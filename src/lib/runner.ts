import { Command, type Child } from '@tauri-apps/plugin-shell'

export interface RunEvent {
  type: 'stdout' | 'stderr' | 'exit' | 'error'
  line?: string
  code?: number | null
  message?: string
}

type Handler = (e: RunEvent) => void

const handlers = new Map<string, Set<Handler>>()
const children = new Map<string, Child>()

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

const IS_WINDOWS = navigator.userAgent.includes('Windows')

/** 按平台为路径加引号：POSIX 用单引号，Windows cmd 用双引号 */
function quotePath(p: string): string {
  if (IS_WINDOWS) return `"${p.replace(/"/g, '""')}"`
  return `'${p.replace(/'/g, `'\\''`)}'`
}

/** 取文件所在目录，兼容 `/` 与 `\`；路径无法解析时返回空串 */
export function dirOf(filePath: string): string {
  const i = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'))
  return i > 0 ? filePath.slice(0, i) : ''
}

/** 切换到执行目录的前缀，例如 `cd 'dir' && `；Windows 用 `cd /d` 以支持跨盘符 */
function cdPrefix(dir: string): string {
  if (!dir) return ''
  return IS_WINDOWS ? `cd /d ${quotePath(dir)} && ` : `cd ${quotePath(dir)} && `
}

/**
 * {file} 占位符替换为引号包裹的路径；无占位符时在末尾追加。
 * 默认先切换到脚本所在目录；托管脚本可通过 execDir 指定其他执行目录。
 */
export function buildCommand(
  template: string,
  filePath: string,
  cdToScriptDir = true,
  execDir?: string,
): string {
  const quoted = quotePath(filePath)
  const t = template.trim()
  const cmd = t.includes('{file}') ? t.split('{file}').join(quoted) : `${t} ${quoted}`
  return `${cdToScriptDir ? cdPrefix(execDir || dirOf(filePath)) : ''}${cmd}`
}

/**
 * plugin-shell 非 raw 模式下按行读取子进程输出，每个 data 事件即一行（不含换行符）。
 * 这里再按 \n 切分一次以兼容块状输出。
 */
function emitLines(runId: string, stream: 'stdout' | 'stderr', chunk: string) {
  for (const line of chunk.split('\n')) emit(runId, { type: stream, line })
}

export async function startRun(
  filePath: string,
  commandTemplate: string,
  cdToScriptDir = true,
  execDir?: string,
): Promise<string> {
  const runId = crypto.randomUUID()
  const cmdLine = buildCommand(commandTemplate, filePath, cdToScriptDir, execDir)
  const cmd = IS_WINDOWS
    ? Command.create('cmd', ['/C', cmdLine])
    : Command.create('sh', ['-c', cmdLine])

  cmd.stdout.on('data', (d) => emitLines(runId, 'stdout', d))
  cmd.stderr.on('data', (d) => emitLines(runId, 'stderr', d))
  cmd.on('close', ({ code }) => {
    children.delete(runId)
    emit(runId, { type: 'exit', code })
  })
  cmd.on('error', (error) => {
    children.delete(runId)
    emit(runId, { type: 'error', message: String(error) })
  })

  const child = await cmd.spawn()
  children.set(runId, child)
  return runId
}

export async function stopRun(runId: string): Promise<void> {
  const child = children.get(runId)
  if (child) await child.kill()
}
