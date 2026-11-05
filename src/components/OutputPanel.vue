<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from 'vue'
import { subscribe } from '../lib/runner'
import { t } from '../lib/i18n'

const props = defineProps<{ runId: string | null }>()

interface Line {
  cls: '' | 'stderr' | 'sys' | 'sys ok' | 'sys err'
  text: string
}

const MAX_LINES = 5000

const body = ref<HTMLElement | null>(null)
const lines = ref<Line[]>([])
let unsubscribe: (() => void) | null = null

function push(line: Line) {
  lines.value.push(line)
  if (lines.value.length > MAX_LINES) lines.value.splice(0, lines.value.length - MAX_LINES)
  nextTick(() => {
    if (body.value) body.value.scrollTop = body.value.scrollHeight
  })
}

watch(
  () => props.runId,
  (id) => {
    unsubscribe?.()
    unsubscribe = null
    if (!id) return // 运行结束后保留已有输出
    lines.value = []
    push({ cls: 'sys', text: `▶ ${t('output.start')}` })
    unsubscribe = subscribe(id, (e) => {
      if (e.type === 'stdout') push({ cls: '', text: e.line ?? '' })
      else if (e.type === 'stderr') push({ cls: 'stderr', text: e.line ?? '' })
      else if (e.type === 'exit') {
        if (e.code === 0) push({ cls: 'sys ok', text: `✔ ${t('output.exitOk')}` })
        else
          push({
            cls: 'sys err',
            text: `✘ ${t('output.exitErr', { code: e.code ?? t('output.unknown') })}`,
          })
      } else if (e.type === 'error') push({ cls: 'sys err', text: t('output.error', { msg: e.message ?? '' }) })
    })
  },
  { immediate: true },
)

onUnmounted(() => unsubscribe?.())

function clear() {
  lines.value = []
}
</script>

<template>
  <div class="output-panel">
    <div class="output-head">
      <span>{{ t('output.title') }}</span>
      <div class="spacer"></div>
      <button class="icon" :title="t('output.clearTip')" @click="clear">{{ t('action.clear') }}</button>
    </div>
    <div ref="body" class="output-body">
      <div v-for="(l, i) in lines" :key="i" class="line" :class="l.cls">{{ l.text }}</div>
      <div v-if="!lines.length" class="line sys">{{ t('output.empty') }}</div>
    </div>
  </div>
</template>
