<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { LOCALES, locale, setLocale, t, type LocaleCode } from '../lib/i18n'

const open = ref(false)
const root = ref<HTMLElement | null>(null)

const currentLabel = computed(
  () => LOCALES.find((l) => l.code === locale.value)?.label ?? locale.value,
)

function toggle() {
  open.value = !open.value
}

function pick(code: LocaleCode) {
  setLocale(code)
  open.value = false
}

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) open.value = false
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  document.addEventListener('keydown', onKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onKeydown)
})
</script>

<template>
  <div ref="root" class="lang-switcher" :class="{ open }">
    <button
      class="lang-btn"
      type="button"
      :title="t('settings.language')"
      :aria-label="t('settings.language')"
      :aria-expanded="open"
      @click="toggle"
    >
      <svg
        class="globe-icon"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
      </svg>
      <span class="lang-label">{{ currentLabel }}</span>
      <svg
        class="chevron"
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

    <Transition name="lang-menu">
      <ul v-if="open" class="lang-menu" role="listbox">
        <li v-for="l in LOCALES" :key="l.code" role="option" :aria-selected="l.code === locale">
          <button
            type="button"
            :class="{ active: l.code === locale }"
            @click="pick(l.code)"
          >
            <span class="check">✓</span>
            <span>{{ l.label }}</span>
          </button>
        </li>
      </ul>
    </Transition>
  </div>
</template>
