# ScriptButler

基于 **Tauri v2 + Vue 3 + TypeScript** 的桌面脚本管理软件。以卡片形式集中管理本地脚本，支持拖拽导入、按最近使用排序、语法高亮编辑，以及可自定义、自动保存的运行命令方案。

## 功能特性

- **卡片式主界面**：脚本以卡片单列纵向排列（默认窗口宽度刚好容纳一张卡片），卡片更高更饱满，显示语言标签、文件名、执行目录、最近使用时间与使用次数。
- **快捷运行**：卡片右侧带有「运行」按钮，一键以当前激活方案运行脚本并进入详情页查看输出；光标悬停在按钮上时显示实际执行的完整命令。
- **拖拽导入**：将脚本文件直接拖入窗口即可新增卡片（也支持右上角「添加脚本」文件选择器）。重复拖入同一文件会将其置顶而不重复添加。
- **搜索筛选**：主界面顶部搜索框按「文件名 → 目录 → 内容」的优先级展示结果；名称与目录由 Rust 在已排序条目上实时分组，内容检索在输入停顿后按需扫描（优先调用系统的 ripgrep / grep，不可用时回退为 Rust 侧逐文件读取），无需预先建立索引。
- **脚本托管**：卡片上的「托管」按钮会将脚本复制到应用数据目录下的 `scripts/`（具体路径由 Tauri 的 `appDataDir` 按平台解析），副本以时间戳命名并保留原扩展名。托管后「托管」按钮图标变为 ❒ 并染橙、不可再点击（悬停提示「已托管」）；卡片只展示实际执行目录，且仅当方案启用「先切换到脚本所在目录再运行」时显示（托管脚本为可配置的执行目录，普通脚本为脚本所在目录）。原脚本名、目录信息与运行方案均被保留，执行目录可在详情页随时更改。
- **MRU 排序**：卡片默认按「最近使用 → 使用次数 → 名称」综合排序。
- **语法高亮编辑**：点击卡片进入详情页，基于 CodeMirror 6 提供高亮与关键词补全，按扩展名自动识别语言（Python / JavaScript / TypeScript / Shell / JSON / YAML / Markdown / HTML / CSS 等）。支持编辑并通过「保存」按钮或 `Ctrl/Cmd + S` 写回文件。
- **自定义运行方案**：详情页顶部提供运行按钮与方案下拉框。每个脚本独立维护一组运行命令方案，支持新建 / 编辑 / 删除；命令模板支持 `{file}` 占位符（缺省时自动在末尾追加脚本路径）。方案内含「先切换到脚本所在目录再运行」开关（默认开启），开启时实际命令形如 `cd '<脚本目录>' && <命令>`，避免脚本依赖当前工作目录时执行出错。方案变更即时自动保存，下次打开可直接下拉选择；首次打开会按语言自动生成默认方案。运行按钮下方会展示当前方案实际执行的完整命令。
- **运行输出面板**：实时流式展示 stdout / stderr（stderr 高亮为红色）、退出码，支持中途停止与清空。
- **持久化**：脚本条目与运行方案保存在应用数据目录的 `scripts.json` 中，重启后恢复。
- **国际化**：内置 9 种界面语言（简体中文 / 繁體中文 / English / 日本語 / 한국어 / Español / Français / Deutsch / Русский），默认跟随系统语言自动选择，可在主界面与详情页右上角随时切换，选择会被记住。

## 技术栈

| 层    | 技术                                               |
| ---- | ------------------------------------------------ |
| 桌面框架 | Tauri v2                                         |
| 前端   | Vue 3 + TypeScript + Vite + Pinia                |
| 编辑器  | CodeMirror 6（`one-dark` 主题）                      |
| 插件   | `tauri-plugin-dialog`                            |

> 架构约定：**业务逻辑在 Rust 侧实现**（持久化、MRU 排序、语言识别、命令拼接、脚本托管、内容检索、子进程运行），前端仅通过 Tauri IPC 命令与事件调用能力并渲染界面。

## 目录结构

```
src/                  # Vue 前端（仅界面渲染与 IPC 调用）
  lib/                # api / language / runner / editor / toast / time / i18n
  locales/            # 9 种语言包（zh-CN 为基准，其余语言与其实全一致）
  stores/scripts.ts   # Pinia store（后端条目列表的薄缓存）
  components/         # CardGrid / ScriptCard / ScriptDetail / RunBar / SchemeDialog / OutputPanel / LangSwitcher
src-tauri/            # Rust 业务层（IPC 命令 + 持久化 + 运行 + 检索）
  src/commands.rs     # 所有 IPC 命令
  src/state.rs        # 条目状态与 scripts.json 读写
  src/exec.rs         # 执行目录解析与命令拼接
  src/runner.rs       # 子进程启动、输出事件、停止
  src/search.rs       # 名称/目录分组与内容检索
  src/language.rs     # 扩展名 → 语言与默认命令
  capabilities/       # core / dialog 权限
```

## 开发

前置要求：Node.js、Rust 工具链，以及 Tauri 的系统依赖（Linux 下需 `webkit2gtk-4.1` 等）。

```bash
npm install        # 安装依赖
npm run tauri dev  # 启动桌面应用（开发模式）
```

## 构建

```bash
npm run build      # 前端类型检查 + 打包（vue-tsc + vite）
npm run tauri build # 构建桌面安装包
```

### 持续集成

推送 `v*` 标签或手动触发 `Build` workflow（`.github/workflows/build.yml`）会在 GitHub Actions 上并行构建各平台安装包：

| 平台      | 产物                            |
| ------- | ----------------------------- |
| Linux   | `.deb` / `.rpm` / `.AppImage` |
| Windows | NSIS 安装包（不构建 MSI）               |
| macOS   | arm64 与 x86_64 的 `.dmg`       |

推送标签时产物会附加到对应的草稿 Release，手动触发时可在 Actions 运行的 Artifacts 中下载。Windows 安装向导的界面语言跟随系统语言（支持应用内置的 9 种语言），系统语言不受支持时回退英语。

## 数据存储

脚本条目与运行方案由 Rust 直接读写应用数据目录下的 `scripts.json`（格式为 `{ "entries": [...] }`）：

- Linux: `~/.local/share/com.scriptbutler.app/scripts.json`
- macOS: `~/Library/Application Support/com.scriptbutler.app/scripts.json`
- Windows: `%APPDATA%/com.scriptbutler.app/scripts.json`

托管脚本副本保存在同一应用数据目录下的 `scripts/` 子目录中，文件名为时间戳 + 原扩展名。
