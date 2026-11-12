(function () {
  "use strict";

  var STORAGE_KEY = "scriptbutler-docs-lang";

  var GITHUB_DOCS_BASE = "https://github.com/UnshodMan/ScriptButler/blob/main/";

  var docLinks = {
    zh: {
      readme: GITHUB_DOCS_BASE + "docs/README.zh-CN.md",
      cli: GITHUB_DOCS_BASE + "docs/README.zh-CN.md#agent-集成cli--skill"
    },
    en: {
      readme: GITHUB_DOCS_BASE + "README.md",
      cli: GITHUB_DOCS_BASE + "README.md#agent-integration-cli--skill"
    }
  };

  var messages = {
    zh: {
      pageTitle: "ScriptButler — 桌面脚本管理工具",
      metaDescription: "ScriptButler 是一款基于 Tauri v2 + Vue 3 + Rust 的桌面脚本管理工具：卡片式管理、一键运行、语法高亮编辑，并内置 sbctl 命令行与 Agent 集成。",
      navFeatures: "功能特性",
      navCli: "命令行",
      navDocs: "文档",
      navDownload: "下载",
      brandLogoAlt: "ScriptButler 图标",
      heroTitle: "把散落的脚本，<br />收拾得<em>井井有条</em>",
      heroLead: "ScriptButler 是一款桌面脚本管理工具。用卡片集中收纳本地脚本，支持拖拽导入、一键运行、语法高亮编辑与自定义运行方案，还可通过内置的 <code>sbctl</code> CLI 让 Agent 直接管理并运行你的脚本。",
      btnDownload: "下载安装包",
      btnSource: "查看源码",
      metaCrossPlatform: "跨平台：Linux / Windows / macOS",
      metaLanguages: "9 种界面语言",
      metaLicense: "MIT 开源协议",
      heroImageAlt: "ScriptButler 主界面：脚本卡片列表与运行输出面板",
      featuresTitle: "为日常脚本而生",
      featuresLead: "从导入、查找、编辑到运行，所有操作都在一个窗口里完成，无需再翻目录、记命令。",
      feature1Title: "卡片式管理",
      feature1Desc: "脚本以卡片纵向排列，语言标签、文件名、执行目录、最近使用时间与次数一目了然。",
      feature2Title: "一键快捷运行",
      feature2Desc: "点击卡片右侧的运行按钮，即以当前激活方案执行脚本，并自动进入详情页查看输出。",
      feature3Title: "拖拽导入",
      feature3Desc: "把脚本文件直接拖进窗口即可加入列表；重复拖入同一文件只会置顶，不会重复添加。",
      feature4Title: "智能搜索",
      feature4Desc: "按「文件名 → 目录 → 内容」的优先级返回结果，内容检索按需扫描，无需预先建立索引。",
      feature5Title: "语法高亮编辑",
      feature5Desc: "基于 CodeMirror 6，自动识别 Python、Shell、JavaScript、JSON、YAML 等语言，支持关键词补全。",
      feature6Title: "自定义运行方案",
      feature6Desc: "每个脚本独立维护命令方案，支持 <code>{file}</code> 占位符、目录切换开关与临时参数，改动即时保存。",
      feature7Title: "实时输出面板",
      feature7Desc: "流式展示 stdout / stderr 与退出码，错误信息高亮显示，支持中途停止与一键清空。",
      feature8Title: "本地持久化",
      feature8Desc: "脚本条目与运行方案保存在本地 <code>scripts.json</code> 中，重启应用后自动恢复。",
      cliTitle: "让 Agent 也能调用你的脚本",
      cliLead: "安装包自带 <code>sbctl</code> 命令行工具，与桌面应用操作同一份数据，无论应用是否运行都能使用。",
      cliPoint1: "添加、托管、列出、搜索、运行与移除脚本",
      cliPoint2: "稳定的 <code>--json</code> 输出和明确的退出码",
      cliPoint3: "<code>sbctl skill --install</code> 一键安装 Agent Skill",
      cliDocs: "查看完整命令文档 →",
      termAdded: "已添加 backup-photos.py",
      termExit: "退出码 0",
      stackTitle: "技术与生态",
      stackLead: "桌面端使用 Tauri v2 构建，业务逻辑沉淀在共享 Rust core 中，前端专注界面渲染。",
      footerDocs: "中文文档",
      footerDownload: "下载",
      langToggleLabel: "切换到英文"
    },
    en: {
      pageTitle: "ScriptButler — Desktop Script Manager",
      metaDescription: "ScriptButler is a desktop script manager built with Tauri v2, Vue 3, and Rust. Organize scripts as cards, run them with one click, edit with syntax highlighting, and manage them from the bundled sbctl CLI and Agent Skill.",
      navFeatures: "Features",
      navCli: "CLI",
      navDocs: "Docs",
      navDownload: "Download",
      brandLogoAlt: "ScriptButler logo",
      heroTitle: "Keep every script<br /><em>organized and ready</em>",
      heroLead: "ScriptButler is a desktop app for managing local scripts. Keep them organized as cards, drag files in to import, run them with one click, edit with syntax highlighting, and define custom run schemes. The bundled <code>sbctl</code> CLI even lets agents manage and run your scripts.",
      btnDownload: "Download",
      btnSource: "View source",
      metaCrossPlatform: "Cross-platform: Linux / Windows / macOS",
      metaLanguages: "9 UI languages",
      metaLicense: "MIT licensed",
      heroImageAlt: "ScriptButler main window showing script cards and the run output panel",
      featuresTitle: "Built for everyday scripts",
      featuresLead: "Import, search, edit, and run — everything happens in one window. No more digging through folders or memorizing commands.",
      feature1Title: "Card-based overview",
      feature1Desc: "Scripts are arranged in a single vertical column of cards, showing the language tag, file name, run directory, last used time, and usage count.",
      feature2Title: "One-click run",
      feature2Desc: "Click Run on a card to execute the script with the active scheme and open the detail page to watch the output.",
      feature3Title: "Drag-and-drop import",
      feature3Desc: "Drag script files straight into the window to add cards. Dropping the same file again moves it to the top instead of adding a duplicate.",
      feature4Title: "Smart search",
      feature4Desc: "Results are ranked by file name, then directory, then content. Content search scans on demand, so there is no index to build in advance.",
      feature5Title: "Syntax-highlighted editing",
      feature5Desc: "Powered by CodeMirror 6 with automatic language detection for Python, Shell, JavaScript, JSON, YAML, and more, plus keyword completion.",
      feature6Title: "Custom run schemes",
      feature6Desc: "Each script keeps its own run command schemes with a <code>{file}</code> placeholder, a working-directory toggle, and per-run arguments. Changes save instantly.",
      feature7Title: "Live output panel",
      feature7Desc: "Streams stdout and stderr in real time with the exit code, highlights errors in red, and supports stopping mid-run and clearing the output.",
      feature8Title: "Local persistence",
      feature8Desc: "Script entries and run schemes are stored in a local <code>scripts.json</code> and restored after a restart.",
      cliTitle: "Let agents call your scripts too",
      cliLead: "The installer ships <code>sbctl</code>, a command-line tool that operates on the same data as the desktop app, whether the app is running or not.",
      cliPoint1: "Add, host, list, search, run, and remove scripts",
      cliPoint2: "Stable <code>--json</code> output and meaningful exit codes",
      cliPoint3: "Install a ready-to-use Agent Skill with <code>sbctl skill --install</code>",
      cliDocs: "Read the full command reference →",
      termAdded: "Added backup-photos.py",
      termExit: "exit code 0",
      stackTitle: "Technology & ecosystem",
      stackLead: "The desktop app is built with Tauri v2. Business logic lives in a shared Rust core while the frontend focuses on UI rendering.",
      footerDocs: "Documentation",
      footerDownload: "Download",
      langToggleLabel: "Switch to Chinese"
    }
  };

  function normalizeLang(value) {
    if (!value) return null;
    var lang = String(value).toLowerCase();
    if (lang.indexOf("zh") === 0) return "zh";
    if (lang.indexOf("en") === 0) return "en";
    return null;
  }

  function readStoredLang() {
    try {
      return normalizeLang(window.localStorage.getItem(STORAGE_KEY));
    } catch (error) {
      return null;
    }
  }

  function storeLang(lang) {
    try {
      window.localStorage.setItem(STORAGE_KEY, lang);
    } catch (error) {
      /* localStorage 不可用时仅影响记忆，不影响当前显示 */
    }
  }

  function detectLang() {
    var localized = normalizeLang(new URLSearchParams(window.location.search).get("lang"));
    if (localized) return localized;

    var stored = readStoredLang();
    if (stored) return stored;

    var candidates = navigator.languages && navigator.languages.length
      ? navigator.languages
      : [navigator.language || ""];

    for (var i = 0; i < candidates.length; i += 1) {
      var lang = normalizeLang(candidates[i]);
      if (lang) return lang;
    }

    return "en";
  }

  function clearLangParam() {
    if (!window.history || !window.history.replaceState) return;
    var url = new URL(window.location.href);
    if (!url.searchParams.has("lang")) return;
    url.searchParams.delete("lang");
    window.history.replaceState(null, "", url.pathname + url.search + url.hash);
  }

  function applyLang(lang) {
    var dict = messages[lang] || messages.zh;

    document.documentElement.lang = lang === "en" ? "en" : "zh-CN";
    document.title = dict.pageTitle;

    var description = document.querySelector('meta[name="description"]');
    if (description) description.setAttribute("content", dict.metaDescription);

    document.querySelectorAll("[data-i18n]").forEach(function (element) {
      var key = element.getAttribute("data-i18n");
      if (dict[key] !== undefined) element.textContent = dict[key];
    });

    document.querySelectorAll("[data-i18n-html]").forEach(function (element) {
      var key = element.getAttribute("data-i18n-html");
      if (dict[key] !== undefined) element.innerHTML = dict[key];
    });

    document.querySelectorAll("[data-i18n-alt]").forEach(function (element) {
      var key = element.getAttribute("data-i18n-alt");
      if (dict[key] !== undefined) element.setAttribute("alt", dict[key]);
    });

    document.querySelectorAll("[data-doc-link]").forEach(function (element) {
      var target = element.getAttribute("data-doc-link");
      if (docLinks[lang] && docLinks[lang][target]) {
        element.setAttribute("href", docLinks[lang][target]);
      }
    });

    var toggle = document.getElementById("lang-toggle");
    if (toggle) {
      toggle.textContent = lang === "zh" ? "EN" : "中文";
      toggle.setAttribute("aria-label", dict.langToggleLabel);
    }
  }

  var currentLang = detectLang();
  applyLang(currentLang);

  var toggle = document.getElementById("lang-toggle");
  if (toggle) {
    toggle.addEventListener("click", function () {
      currentLang = currentLang === "zh" ? "en" : "zh";
      storeLang(currentLang);
      clearLangParam();
      applyLang(currentLang);
    });
  }
})();
