<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { confirm, open } from '@tauri-apps/plugin-dialog'
import { useScriptsStore } from '../stores/scripts'
import { t } from '../lib/i18n'
import { toast } from '../lib/toast'
import SchemeDialog from './SchemeDialog.vue'
import type { RunScheme, ScriptEntry } from '../types'

const props = defineProps<{
  entry: ScriptEntry
  running: boolean
}>()

const emit = defineEmits<{
  run: [scheme: RunScheme, args: string]
  stop: []
}>()

const store = useScriptsStore()

const activeScheme = computed(
  () =>
    props.entry.schemes.find((s) => s.id === props.entry.activeSchemeId) ??
    props.entry.schemes[0] ??
    null,
)

const dialogOpen = ref(false)
const dialogTitle = ref('')
const dialogInitial = ref<{ name: string; command: string; cdToScriptDir: boolean } | null>(null)
let editingId: string | null = null

const menuOpen = ref(false)
const root = ref<HTMLElement | null>(null)

/** 实际执行目录：由 Rust 解析后随条目下发 */
const execDir = computed(() => props.entry.execDir ?? '')

/** 当前方案实际执行的完整命令（含可能的 cd 切换目录） */
const effectiveCommand = computed(() => props.entry.effectiveCommand ?? '')

/** 本次运行的临时参数，仅追加到命令行末尾，不写回方案 */
const extraArgs = ref('')

/** 预览：实际命令 + 本次临时参数 */
const previewCommand = computed(() => {
  const args = extraArgs.value.trim()
  return args ? `${effectiveCommand.value} ${args}` : effectiveCommand.value
})

/** 托管脚本：随时更改执行目录 */
async function pickDir() {
  const picked = await open({
    directory: true,
    multiple: false,
    title: t('runbar.pickDirTitle'),
    defaultPath: execDir.value || undefined,
  })
  if (typeof picked === 'string') {
    await store.setWorkDir(props.entry.path, picked)
    toast(t('runbar.dirChanged'))
  }
}

function toggleMenu() {
  if (props.entry.schemes.length === 0) return
  menuOpen.value = !menuOpen.value
}

function pickScheme(id: string) {
  store.setActiveScheme(props.entry.path, id)
  menuOpen.value = false
}

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) menuOpen.value = false
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') menuOpen.value = false
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onKeydown)
})

function openCreate() {
  editingId = null
  dialogTitle.value = t('runbar.newTitle')
  dialogInitial.value = null
  dialogOpen.value = true
}

function openEdit() {
  if (!activeScheme.value) return
  editingId = activeScheme.value.id
  dialogTitle.value = t('runbar.editTitle')
  dialogInitial.value = {
    name: activeScheme.value.name,
    command: activeScheme.value.command,
    cdToScriptDir: activeScheme.value.cdToScriptDir,
  }
  dialogOpen.value = true
}

function onDialogSave(data: { name: string; command: string; cdToScriptDir: boolean }) {
  if (editingId) {
    store.updateScheme(props.entry.path, editingId, data.name, data.command, data.cdToScriptDir)
  } else {
    store.createScheme(props.entry.path, data.name, data.command, data.cdToScriptDir)
  }
  dialogOpen.value = false
}

async function removeActive() {
  const target = activeScheme.value
  if (!target) return
  const ok = await confirm(t('runbar.deleteConfirm', { name: target.name }), {
    title: t('runbar.deleteTitle'),
    kind: 'warning',
  })
  if (!ok) return
  store.deleteScheme(props.entry.path, target.id)
}

function onRunClick() {
  if (props.running) {
    emit('stop')
    return
  }
  if (!activeScheme.value) {
    toast(t('runbar.needScheme'), 'error')
    return
  }
  emit('run', activeScheme.value, extraArgs.value.trim())
}
</script>

<template>
  <div class="run-bar">
    <div class="run-bar-row">
      <div ref="root" class="scheme-select" :class="{ open: menuOpen }">
        <button
          type="button"
          class="scheme-trigger"
          :disabled="entry.schemes.length === 0"
          :title="t('runbar.selectScheme')"
          :aria-label="t('runbar.selectScheme')"
          :aria-expanded="menuOpen"
          @click="toggleMenu"
        >
          <span class="scheme-trigger-label">
            <template v-if="activeScheme">
              <span class="scheme-name">{{ activeScheme.name }}</span>
              <span class="scheme-cmd">{{ activeScheme.command }}</span>
            </template>
            <template v-else>{{ t('runbar.noScheme') }}</template>
          </span>
          <svg
            class="caret"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
          >
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>

        <Transition name="scheme-menu">
          <ul v-if="menuOpen && entry.schemes.length > 0" class="scheme-menu" role="listbox">
            <li
              v-for="s in entry.schemes"
              :key="s.id"
              role="option"
              :aria-selected="s.id === activeScheme?.id"
            >
              <button
                type="button"
                :class="{ active: s.id === activeScheme?.id }"
                @click="pickScheme(s.id)"
              >
                <span class="scheme-name">{{ s.name }}</span>
                <span class="scheme-cmd">{{ s.command }}</span>
              </button>
            </li>
          </ul>
        </Transition>
      </div>

      <button class="icon" :title="t('runbar.newScheme')" @click="openCreate">＋</button>
      <button class="icon" :title="t('runbar.editScheme')" :disabled="!activeScheme" @click="openEdit">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M12 20h9" />
          <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
        </svg>
      </button>
      <button
        class="icon danger"
        :title="t('runbar.deleteScheme')"
        :disabled="!activeScheme"
        @click="removeActive"
      >
        🗑
      </button>

      <div class="spacer"></div>

      <button v-if="running" class="run-btn danger" :title="t('action.stop')" @click="onRunClick">■</button>
      <button v-else class="run-btn" :title="t('action.run')" @click="onRunClick">▶</button>
    </div>

    <div v-if="entry.managed || activeScheme" class="run-meta">
      <template v-if="entry.managed">
        <span class="label">{{ t('runbar.execDir') }}</span>
        <div class="value">
          <code :title="execDir">{{ execDir }}</code>
          <button class="dir-btn" :title="t('runbar.changeDir')" @click="pickDir">
            {{ t('runbar.changeDir') }}
          </button>
        </div>
      </template>

      <template v-if="activeScheme">
        <span class="label">{{ t('runbar.args') }}</span>
        <div class="value">
          <input
            v-model="extraArgs"
            class="args-input"
            type="text"
            :placeholder="t('runbar.argsPlaceholder')"
            :title="t('runbar.argsTip')"
            spellcheck="false"
            autocomplete="off"
          />
        </div>

        <span class="label">{{ t('runbar.effectiveCommand') }}</span>
        <div class="value" :title="previewCommand">
          <code>{{ previewCommand }}</code>
        </div>
      </template>
    </div>

    <SchemeDialog
      v-if="dialogOpen"
      :title="dialogTitle"
      :dir="execDir"
      :initial="dialogInitial"
      @save="onDialogSave"
      @cancel="dialogOpen = false"
    />
  </div>
</template>
