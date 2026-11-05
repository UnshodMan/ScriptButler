<script setup lang="ts">
import { ref } from 'vue'
import { t } from '../lib/i18n'

const props = defineProps<{
  title: string
  /** 脚本所在目录，用于在提示中展示将要切换到的目录 */
  dir: string
  initial: { name: string; command: string; cdToScriptDir: boolean } | null
}>()

const emit = defineEmits<{
  save: [data: { name: string; command: string; cdToScriptDir: boolean }]
  cancel: []
}>()

const name = ref(props.initial?.name ?? '')
const command = ref(props.initial?.command ?? '')
const cdToScriptDir = ref(props.initial?.cdToScriptDir ?? true)

const valid = () => name.value.trim().length > 0 && command.value.trim().length > 0

function submit() {
  if (!valid()) return
  emit('save', {
    name: name.value.trim(),
    command: command.value.trim(),
    cdToScriptDir: cdToScriptDir.value,
  })
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('cancel')">
    <div class="modal">
      <h3>{{ title }}</h3>
      <div class="field">
        <label>{{ t('dialog.schemeName') }}</label>
        <input v-model="name" :placeholder="t('dialog.namePlaceholder')" @keydown.enter="submit" />
      </div>
      <div class="field">
        <label>{{ t('dialog.runCommand') }}</label>
        <input
          v-model="command"
          :placeholder="t('dialog.commandPlaceholder')"
          @keydown.enter="submit"
        />
      </div>
      <div class="field check">
        <label>
          <input v-model="cdToScriptDir" type="checkbox" />
          <span>{{ t('dialog.cdToScriptDir') }}</span>
        </label>
      </div>
      <div class="hint">
        {{ t('dialog.hint', { ph: '{file}' }) }}
      </div>
      <div class="hint">
        {{ t('dialog.cdHint', { dir }) }}
      </div>
      <div class="actions">
        <button @click="emit('cancel')">{{ t('action.cancel') }}</button>
        <button class="primary" :disabled="!valid()" @click="submit">{{ t('action.save') }}</button>
      </div>
    </div>
  </div>
</template>
