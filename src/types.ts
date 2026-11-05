export interface RunScheme {
  id: string
  name: string
  /** 命令模板，支持 {file} 占位符；缺省时自动在末尾追加脚本路径 */
  command: string
  /** 执行前是否先切换到脚本所在目录（默认开启） */
  cdToScriptDir: boolean
}

export interface ScriptEntry {
  /** 绝对路径，唯一主键 */
  path: string
  name: string
  /** 语言 id，按扩展名识别 */
  lang: string
  /** ms 时间戳，MRU 主排序键 */
  lastUsedAt: number
  /** 使用次数，次级排序键 */
  useCount: number
  schemes: RunScheme[]
  activeSchemeId: string | null
  /** 由 ScriptButler 托管：脚本文件副本位于应用数据目录下的 scripts/ */
  managed?: boolean
  /** 托管前的原始文件路径，用于保留原文件名与目录信息 */
  sourcePath?: string
  /** 托管脚本的执行目录，可随时修改；默认取原始脚本所在目录 */
  workDir?: string
}
