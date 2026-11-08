<script setup lang="ts">
import { computed, ref } from 'vue'
import { confirm } from '@tauri-apps/plugin-dialog'
import { IconPlayerPlay, IconCopyCheck, IconCopy, IconX } from '@tabler/icons-vue'
import { useScriptsStore } from '../stores/scripts'
import { languageLabel } from '../lib/language'
import { relTime } from '../lib/time'
import { t } from '../lib/i18n'
import { toast } from '../lib/toast'
import type { ScriptEntry } from '../types'

const props = defineProps<{ entry: ScriptEntry }>()
const emit = defineEmits<{ open: [path: string]; run: [path: string] }>()

const store = useScriptsStore()

const activeScheme = computed(
  () =>
    props.entry.schemes.find((s) => s.id === props.entry.activeSchemeId) ??
    props.entry.schemes[0] ??
    null,
)

/**
 * 卡片展示的执行目录：仅在启用“先切换到脚本所在目录再运行”时有意义。
 * 具体目录由 Rust 解析后随条目下发。
 */
const execDir = computed(() =>
  activeScheme.value?.cdToScriptDir ? props.entry.execDir : null,
)

/** 悬停运行按钮时展示的实际执行命令 */
const runCommand = computed(() =>
  activeScheme.value ? (props.entry.effectiveCommand ?? t('card.noScheme')) : t('card.noScheme'),
)

const managing = ref(false)

async function manageEntry() {
  if (managing.value) return
  managing.value = true
  try {
    const r = await store.manage(props.entry.path)
    if (r.ok && r.entry) toast(t('card.managedDone', { path: r.entry.path }))
    else if (r.message) toast(r.message, 'error')
  } finally {
    managing.value = false
  }
}

async function removeEntry() {
  const ok = await confirm(t('card.removeConfirm', { name: props.entry.name }), {
    title: t('card.removeTitle'),
    kind: 'warning',
  })
  if (ok) await store.remove(props.entry.path)
}
</script>

<template>
  <div class="card" @click="emit('open', entry.path)">
    <div class="card-head">
      <span class="card-name" :title="entry.name">{{ entry.name }}</span>
      <span class="lang-tag">{{ languageLabel(entry.lang) }}</span>
      <button
        class="run-btn"
        :title="runCommand"
        :disabled="!activeScheme"
        @click.stop="emit('run', entry.path)"
      >
        <IconPlayerPlay class="run-icon" :size="11" />
        <span>{{ t('action.run') }}</span>
      </button>
    </div>

    <div v-if="execDir" class="card-path" :title="execDir">
      {{ t('card.execDir', { dir: execDir }) }}
    </div>

    <div class="card-foot">
      <div class="card-stats">
        <span>{{ t('card.lastUsed', { time: relTime(entry.lastUsedAt) }) }}</span>
        <span class="stat-dot">·</span>
        <span>{{ t('card.useCount', { n: entry.useCount }) }}</span>
      </div>
      <div class="card-actions">
        <button
          class="manage-btn"
          :class="{ managed: entry.managed }"
          :title="entry.managed ? t('card.managed') : t('card.manageTip')"
          :aria-label="entry.managed ? t('card.managed') : t('card.manageBtn')"
          :disabled="entry.managed || managing"
          @click.stop="manageEntry"
        >
          <IconCopyCheck v-if="entry.managed" :size="15" />
          <IconCopy v-else :size="15" />
        </button>
        <button class="remove-btn" :title="t('card.removeBtn')" @click.stop="removeEntry">
          <IconX :size="15" />
        </button>
      </div>
    </div>
  </div>
</template>
