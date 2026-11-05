import { ref } from 'vue'
import { zhCN, type Messages } from '../locales/zh-CN'
import { zhTW } from '../locales/zh-TW'
import { en } from '../locales/en'
import { ja } from '../locales/ja'
import { ko } from '../locales/ko'
import { es } from '../locales/es'
import { de } from '../locales/de'
import { fr } from '../locales/fr'
import { ru } from '../locales/ru'

export const LOCALES = [
  { code: 'en', label: 'English' },
  { code: 'zh-CN', label: '简体中文' },
  { code: 'zh-TW', label: '繁體中文' },
  { code: 'ja', label: '日本語' },
  { code: 'ko', label: '한국어' },
  { code: 'es', label: 'Español' },
  { code: 'fr', label: 'Français' },
  { code: 'de', label: 'Deutsch' },
  { code: 'ru', label: 'Русский' },
] as const

export type LocaleCode = (typeof LOCALES)[number]['code']

const MESSAGES: Record<LocaleCode, Messages> = {
  en,
  'zh-CN': zhCN,
  'zh-TW': zhTW,
  ja,
  ko,
  es,
  fr,
  de,
  ru,
}

const STORAGE_KEY = 'scriptbutler.locale'

/** 依据系统语言匹配界面语言；未匹配到支持语言时回退英文 */
function detectLocale(): LocaleCode {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved && saved in MESSAGES) return saved as LocaleCode

  const candidates = navigator.languages?.length ? navigator.languages : [navigator.language]
  for (const lang of candidates) {
    const tag = lang.toLowerCase()
    if (tag.startsWith('zh')) {
      // zh-TW / zh-Hant / zh-HK 等繁体变体映射到繁体中文
      return /tw|hk|mo|hant/.test(tag) ? 'zh-TW' : 'zh-CN'
    }
    const prefix = tag.split('-')[0]
    const hit = LOCALES.find((l) => l.code === prefix)
    if (hit) return hit.code
  }
  return 'en'
}

export const locale = ref<LocaleCode>(detectLocale())

document.documentElement.lang = locale.value

/**
 * 翻译查找：当前语言 → 英文兜底 → key 本身。
 * 在模板 / computed 中调用时会追踪 locale，切换语言即自动重渲染。
 */
export function t(key: keyof Messages, params?: Record<string, string | number>): string {
  let msg: string = MESSAGES[locale.value][key] ?? MESSAGES.en[key] ?? key
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      msg = msg.split(`{${k}}`).join(String(v))
    }
  }
  return msg
}

export function setLocale(code: LocaleCode) {
  locale.value = code
  localStorage.setItem(STORAGE_KEY, code)
  document.documentElement.lang = code
}
