<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { EditorView } from '@codemirror/view'
import { useScriptsStore } from '../stores/scripts'
import * as api from '../lib/api'
import { createEditor } from '../lib/editor'
import { startRun, stopRun, subscribe } from '../lib/runner'
import { t } from '../lib/i18n'
import { toast } from '../lib/toast'
import RunBar from './RunBar.vue'
import OutputPanel from './OutputPanel.vue'
import type { RunScheme } from '../types'

const props = defineProps<{ path: string; autorun?: boolean }>()
const emit = defineEmits<{ back: [] }>()

const store = useScriptsStore()

const entry = computed(() => store.byPath(props.path))
const editorHost = ref<HTMLElement | null>(null)
const dirty = ref(false)
const loadError = ref<string | null>(null)
const runId = ref<string | null>(null)

let view: EditorView | null = null
const savedContent = ref('')

async function save() {
  if (!view) return
  const content = view.state.doc.toString()
  try {
    await api.writeScript(props.path, content)
    savedContent.value = content
    dirty.value = false
    toast(t('detail.saved'))
  } catch (e) {
    toast(api.errorText(e), 'error')
  }
}

async function run(scheme: RunScheme, args = '') {
  await store.markUsed(props.path)
  try {
    runId.value = await startRun(props.path, scheme.id, args)
  } catch (e) {
    toast(api.errorText(e), 'error')
  }
}

async function stop() {
  if (runId.value) await stopRun(runId.value)
}

// 运行结束（正常退出/出错）后复位运行状态
watch(runId, (id) => {
  if (!id) return
  const un = subscribe(id, (e) => {
    if (e.type === 'exit' || e.type === 'error') {
      runId.value = null
      un()
    }
  })
})

onMounted(async () => {
  await store.markUsed(props.path)

  // 首次打开且无方案时，按语言生成默认方案（由 Rust 兜底）
  await store.ensureDefaultScheme(props.path, t('detail.defaultScheme'))

  try {
    savedContent.value = await api.readScript(props.path)
  } catch (err) {
    loadError.value = api.errorText(err)
    return
  }

  if (editorHost.value && entry.value) {
    view = createEditor(editorHost.value, {
      doc: savedContent.value,
      langId: entry.value.lang,
      baseline: savedContent,
      onSave: save,
      onDirtyChange: (d) => (dirty.value = d),
    })
  }

  // 从卡片快捷运行进入：自动执行当前激活方案
  if (props.autorun) {
    const e = store.byPath(props.path)
    const scheme =
      e?.schemes.find((s) => s.id === e.activeSchemeId) ?? e?.schemes[0] ?? null
    if (scheme) await run(scheme)
  }
})

onUnmounted(() => {
  view?.destroy()
  view = null
})
</script>

<template>
  <div class="detail-page">
    <div class="detail-topbar">
      <button @click="emit('back')">← {{ t('action.back') }}</button>
      <span class="file-name">{{ entry?.name }}</span>
      <span class="file-path">{{ path }}</span>
      <span v-if="dirty" class="dirty-dot" :title="t('detail.dirtyTip')"></span>
      <button class="primary" :disabled="!dirty" @click="save">{{ t('action.save') }}</button>
    </div>

    <RunBar
      v-if="entry"
      :entry="entry"
      :running="runId !== null"
      @run="run"
      @stop="stop"
    />

    <div class="editor-host">
      <div v-if="loadError" class="empty-state">{{ loadError }}</div>
      <div v-else ref="editorHost" style="height: 100%"></div>
    </div>

    <OutputPanel :run-id="runId" />
  </div>
</template>
