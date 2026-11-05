<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { useScriptsStore } from '../stores/scripts'
import { t } from '../lib/i18n'
import { toast } from '../lib/toast'
import ScriptCard from './ScriptCard.vue'
import LangSwitcher from './LangSwitcher.vue'

const emit = defineEmits<{ open: [path: string]; run: [path: string] }>()

const store = useScriptsStore()

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
      <span class="count">{{ t('grid.count', { n: store.sorted.length }) }}</span>
      <div class="spacer"></div>
      <LangSwitcher />
      <button class="primary" @click="pickFiles">+ {{ t('action.addScript') }}</button>
    </header>

    <div class="grid-scroll">
      <div v-if="store.sorted.length" class="card-grid">
        <ScriptCard
          v-for="entry in store.sorted"
          :key="entry.path"
          :entry="entry"
          @open="emit('open', $event)"
          @run="emit('run', $event)"
        />
      </div>

      <div v-else class="empty-state">
        <div class="big">📜</div>
        <div>{{ t('grid.empty.title') }}</div>
        <div>{{ t('grid.empty.hint') }}</div>
      </div>
    </div>
  </div>
</template>
