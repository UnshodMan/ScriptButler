<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useScriptsStore } from './stores/scripts'
import CardGrid from './components/CardGrid.vue'
import ScriptDetail from './components/ScriptDetail.vue'
import ToastHost from './components/ToastHost.vue'
import { toast } from './lib/toast'

const store = useScriptsStore()
const openPath = ref<string | null>(null)
const openAutoRun = ref(false)

function openCard(path: string) {
  openAutoRun.value = false
  openPath.value = path
}

function runCard(path: string) {
  openAutoRun.value = true
  openPath.value = path
}

let unlistenDrop: (() => void) | null = null
let unlistenFocus: (() => void) | null = null
const prevent = (e: DragEvent) => e.preventDefault()

onMounted(async () => {
  await store.load()

  // 阻止 webview 默认的拖放行为（防止打开文件）
  window.addEventListener('dragover', prevent)
  window.addEventListener('drop', prevent)

  // Tauri 原生文件拖拽：获取真实文件路径
  unlistenDrop = await getCurrentWindow().onDragDropEvent(async (e) => {
    if (e.payload.type !== 'drop') return
    for (const p of e.payload.paths) {
      const r = await store.addByPath(p)
      if (!r.ok && r.message) toast(r.message, 'error')
    }
  })

  // 窗口重新获得焦点时重载，感知 CLI 等外部进程对 scripts.json 的修改
  unlistenFocus = await getCurrentWindow().onFocusChanged(({ payload }) => {
    if (payload && store.loaded) void store.load()
  })
})

onUnmounted(() => {
  window.removeEventListener('dragover', prevent)
  window.removeEventListener('drop', prevent)
  unlistenDrop?.()
  unlistenFocus?.()
})
</script>

<template>
  <div class="app">
    <ScriptDetail
      v-if="openPath"
      :key="openPath"
      :path="openPath"
      :autorun="openAutoRun"
      @back="openPath = null"
    />
    <CardGrid v-else @open="openCard" @run="runCard" />
    <ToastHost />
  </div>
</template>
