<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { IconSearch, IconX } from '@tabler/icons-vue'
import { useScriptsStore } from '../stores/scripts'
import * as api from '../lib/api'
import { t } from '../lib/i18n'
import { toast } from '../lib/toast'
import ScriptCard from './ScriptCard.vue'
import LangSwitcher from './LangSwitcher.vue'
import type { ScriptEntry, SearchGroups } from '../types'

const emit = defineEmits<{ open: [path: string]; run: [path: string] }>()

const store = useScriptsStore()

const query = ref('')
const keyword = computed(() => query.value.trim().toLowerCase())
const groups = ref<SearchGroups | null>(null)
const contentPaths = ref<Set<string>>(new Set())
const searching = ref(false)

let searchTimer: number | undefined
let searchSeq = 0

/** 结果顺序：文件名匹配 → 目录匹配 → 内容匹配，组内保持 MRU 顺序 */
const results = computed<ScriptEntry[]>(() => {
  const g = groups.value
  if (!keyword.value || !g) return store.entries
  const byPath = new Map(store.entries.map((e) => [e.path, e]))
  const out: ScriptEntry[] = []
  const push = (path: string) => {
    const entry = byPath.get(path)
    if (entry) out.push(entry)
  }
  g.byName.forEach(push)
  g.byDir.forEach(push)
  for (const path of g.pending) {
    if (contentPaths.value.has(path)) push(path)
  }
  return out
})

watch(keyword, async (q) => {
  searchSeq += 1
  const seq = searchSeq
  window.clearTimeout(searchTimer)
  contentPaths.value = new Set()
  groups.value = null

  if (!q) {
    searching.value = false
    return
  }

  // 文件名与目录匹配由 Rust 在已排序条目上分组返回
  const found = await api.searchScripts(q)
  if (seq !== searchSeq) return
  groups.value = found
  if (!found.pending.length) {
    searching.value = false
    return
  }

  searching.value = true
  // 输入停顿后再检索内容，避免每次按键都拉起外部进程
  searchTimer = window.setTimeout(async () => {
    const paths = await api.searchContents(found.pending, q)
    if (seq !== searchSeq) return
    contentPaths.value = new Set(paths)
    searching.value = false
  }, 250)
})

onUnmounted(() => window.clearTimeout(searchTimer))

function clearSearch() {
  query.value = ''
}

async function pickFiles() {
  const selected = await open({ multiple: true, title: t('grid.pickTitle') })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  for (const p of paths) {
    const r = await store.addByPath(p)
    if (!r.ok && r.message) toast(r.message, 'error')
  }
}
</script>

<template>
  <div class="grid-page">
    <header class="grid-header">
      <h1>ScriptButler</h1>
      <span class="count">{{ t('grid.count', { n: store.entries.length }) }}</span>
      <div class="spacer"></div>
      <LangSwitcher />
      <button class="primary" @click="pickFiles">+ {{ t('action.addScript') }}</button>
    </header>

    <div class="grid-search">
      <div class="search-field">
        <IconSearch class="search-icon" :size="15" />
        <input
          v-model="query"
          type="text"
          :placeholder="t('search.placeholder')"
          :aria-label="t('search.placeholder')"
          autocomplete="off"
          spellcheck="false"
          @keydown.esc="clearSearch"
        />
        <span v-if="keyword" class="search-status">
          {{ searching ? t('search.searching') : t('search.results', { n: results.length }) }}
        </span>
        <button
          v-if="query"
          class="search-clear"
          :title="t('search.clear')"
          :aria-label="t('search.clear')"
          @click="clearSearch"
        >
          <IconX :size="14" />
        </button>
      </div>
    </div>

    <div class="grid-scroll">
      <div v-if="results.length" class="card-grid">
        <ScriptCard
          v-for="entry in results"
          :key="entry.path"
          :entry="entry"
          @open="emit('open', $event)"
          @run="emit('run', $event)"
        />
      </div>

      <div v-else-if="keyword" class="empty-state">
        <div class="big">🔍</div>
        <div>
          {{ searching ? t('search.searching') : t('search.empty.title', { q: query.trim() }) }}
        </div>
        <div v-if="!searching">{{ t('search.empty.hint') }}</div>
      </div>

      <div v-else class="empty-state">
        <div class="big">📜</div>
        <div>{{ t('grid.empty.title') }}</div>
        <div>{{ t('grid.empty.hint') }}</div>
      </div>
    </div>
  </div>
</template>
