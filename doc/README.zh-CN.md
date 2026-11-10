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
- **Agent 集成**：随桌面应用一起分发 `sbctl` CLI，Agent 与终端可使用稳定的 `--json` 输出添加、管理、列出、搜索、运行与移除脚本，`sbctl skill --install <dir>` 可安装开箱即用的 Agent Skill。详见 [Agent 集成（CLI + Skill）](#agent-集成-cli--skill)。

## Agent 集成（CLI + Skill）

安装包会附带 `sbctl`——一个操作与应用相同的 `scripts.json` 的 Rust CLI。无论应用是否在运行都可以使用。

### 二进制位置与 PATH

| 平台              | 二进制位置                                   | PATH                                                                                              |
| --------------- | --------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Linux（deb/rpm）  | `/usr/bin/sbctl`                        | 自动添加                                                                                              |
| Windows（NSIS）   | `<安装目录>\sbctl.exe`                      | 在安装器中勾选「Add sbctl to PATH」，或运行 `sbctl env add`                                                    |
| macOS（dmg）      | `ScriptButler.app/Contents/MacOS/sbctl` | 运行 `/Applications/ScriptButler.app/Contents/MacOS/sbctl env add`；如提示需将 `~/.local/bin` 加入 PATH 则照做 |
| Linux（AppImage） | AppImage 内部                             | 不对外暴露；请使用 deb/rpm 包                                                                               |

Windows 下 `sbctl env add` 会将可执行文件所在目录加入当前用户的 PATH（加 `--system` 则加入机器级 PATH）。macOS/Linux 下会创建 `~/.local/bin/sbctl`，除非该目录已在 PATH 中。`sbctl env remove` 撤销该操作。

### 命令

| 命令                                                                                          | 说明                         |
| ------------------------------------------------------------------------------------------- | -------------------------- |
| `sbctl add <path> [--name N] [--command TPL] [--scheme-name N] [--json]`                    | 就地注册脚本；重复添加仅更新 MRU         |
| `sbctl manage <path> [--name N] [--workdir DIR] [--command TPL] [--scheme-name N] [--json]` | 将脚本复制到托管目录；对已注册与未注册文件均可用   |
| `sbctl remove <name\|path> [--json]`                                                        | 移除条目；绝不删除文件                |
| `sbctl list [--query KW] [--json]`                                                          | 按 MRU 顺序列出条目               |
| `sbctl show <name\|path> [--json]`                                                          | 展示条目、其方案与生效命令              |
| `sbctl search <keyword> [--content] [--json]`                                               | 搜索名称、目录，可选搜索内容             |
| `sbctl run <name\|path> [--scheme ID\|NAME] [--json]`                                       | 运行激活或指定的方案                 |
| `sbctl path [--json]`                                                                       | 打印数据目录、store 路径、托管目录与可执行文件 |
| `sbctl env add\|remove\|status [--system] [--json]`                                         | 管理 PATH                    |
| `sbctl skill [--install <skills-root>]`                                                     | 打印或安装 Agent Skill          |

`<name|path>` 依次按绝对路径、托管脚本的源路径、文件名（不区分大小写）解析。名称重复时返回 `ambiguous` 错误并附候选路径。

### JSON 输出与退出码

传入 `--json` 获得机器可读输出：

```json
{ "ok": true, "data": [ ... ] }
{ "ok": false, "error": { "code": "not_found", "message": "..." } }
```

`add`/`manage` 还会返回 `created` 与 `copiedTo`；`run` 返回 `{ exitCode, stdout, stderr }`。

| 退出码   | 含义                     |
| ----- | ---------------------- |
| 0     | 成功                     |
| 1     | 运行时错误（I/O、spawn、托管）    |
| 2     | 未找到、引用有歧义或用法错误         |
| 3     | `scripts.json` 无效      |
| 脚本返回码 | `sbctl run` 返回脚本自身的退出码 |

### Agent Skill

仓库包含 `skills/script-butler/SKILL.md`。安装到你的 Agent 技能目录：

```bash
sbctl skill --install ~/.claude/skills   # 替换为你的 Agent 技能根目录
```

`sbctl skill` 会将同一文档打印到 stdout。

### 数据同步

CLI 与应用读写同一个 `scripts.json`。当文件在磁盘上发生变化时，应用会在每条命令前重新加载，并在窗口重新获得焦点时刷新卡片列表。若两个进程同时写入，后写入者生效。

## 技术栈

| 层    | 技术                                 |
| ---- | ---------------------------------- |
| 桌面框架 | Tauri v2                           |
| 前端   | Vue 3 + TypeScript + Vite + Pinia  |
| 编辑器  | CodeMirror 6（`one-dark` 主题）        |
| 插件   | `tauri-plugin-dialog`              |
| CLI  | Rust（`scriptbutler-core` + `clap`） |

> 架构约定：**业务逻辑位于共享的 Rust core crate**（`src-tauri/core`），由桌面应用与 `sbctl` CLI 共用。前端仅通过 Tauri IPC 命令与事件调用这些能力并渲染界面。

## 目录结构

```
src/                  # Vue 前端（仅界面渲染与 IPC 调用）
  lib/                # api / language / runner / editor / toast / time / i18n
  locales/            # 9 种语言包（zh-CN 为基准，其余语言与基准保持同步）
  stores/scripts.ts   # Pinia store（后端条目列表的薄缓存）
  components/         # CardGrid / ScriptCard / ScriptDetail / RunBar / SchemeDialog / OutputPanel / LangSwitcher
src-tauri/            # Tauri 桌面应用（IPC 命令 + 子进程事件）
  src/commands.rs     # 基于共享 core 的薄 IPC 命令封装
  src/state.rs        # 内存条目 + scripts.json 重载/保存
  src/runner.rs       # 子进程启动、输出事件、停止
  core/               # scriptbutler-core: models / store / ops / exec / search / language
  cli/                # scriptbutler-cli: sbctl 命令行界面
  nsis/               # 带 sbctl PATH 选项的 Windows 安装包模板
  capabilities/       # core / dialog 权限
skills/               # Agent Skill（script-butler/SKILL.md）
scripts/              # build-cli.mjs / bump-version.mjs
```

## 开发

前置要求：Node.js、Rust 工具链，以及 Tauri 的系统依赖（Linux 下需 `webkit2gtk-4.1` 等）。

```bash
npm install         # 安装依赖
npm run build:cli   # 构建 sbctl sidecar（tauri dev/build 前也会自动执行）
npm run tauri dev   # 启动桌面应用（开发模式）
```

## 构建

```bash
npm run build        # 前端类型检查 + 打包（vue-tsc + vite）
npm run build:cli    # 仅构建 sbctl sidecar
npm run tauri build  # 构建桌面安装包（内置 sbctl）
```

打包器通过 `bundle.externalBin` 读取 `src-tauri/binaries/sbctl-<target-triple>`。`beforeBuildCommand` 会针对 Tauri 当前使用的目标三元组构建它（`TAURI_ENV_TARGET_TRIPLE`），CI 也会为矩阵中的每个目标预先构建。

### 版本号升级

应用版本在 `package.json`、`package-lock.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 与 `src-tauri/Cargo.lock` 中保持一致。`[workspace.package]` 中的工作区版本由 `scriptbutler-core` 与 `scriptbutler-cli` 继承；脚本会更新该版本以及三处 lockfile 条目。一步完成全部升级：

```bash
npm run version:bump -- patch   # 0.1.0 -> 0.1.1
npm run version:bump -- minor   # 0.1.0 -> 0.2.0
npm run version:bump -- major   # 0.1.0 -> 1.0.0
npm run version:bump -- 0.2.3   # 指定具体版本
```

### 持续集成

推送 `v*` 标签或手动触发 `Build` workflow（`.github/workflows/build.yml`）会在 GitHub Actions 上并行构建各平台安装包：

| 平台      | 产物                            |
| ------- | ----------------------------- |
| Linux   | `.deb` / `.rpm` / `.AppImage` |
| Windows | NSIS 安装包（不构建 MSI）             |
| macOS   | arm64 与 x86\_64 的 `.dmg`      |

推送标签时产物会附加到对应的草稿 Release，手动触发时可在 Actions 运行的 Artifacts 中下载。Windows 安装向导的界面语言跟随系统语言（支持应用内置的 9 种语言），系统语言不受支持时回退英语。

## 数据存储

脚本条目与运行方案由共享的 Rust core 读写应用数据目录下的 `scripts.json`（格式为 `{ "entries": [...] }`）。`sbctl` CLI 使用相同路径；可通过 `SCRIPTBUTLER_DATA_DIR` 覆盖 CLI 的数据目录（便于测试）：

- Linux: `~/.local/share/com.scriptbutler.app/scripts.json`
- macOS: `~/Library/Application Support/com.scriptbutler.app/scripts.json`
- Windows: `%APPDATA%/com.scriptbutler.app/scripts.json`

托管脚本副本保存在同一应用数据目录下的 `scripts/` 子目录中，文件名为时间戳 + 原扩展名。
