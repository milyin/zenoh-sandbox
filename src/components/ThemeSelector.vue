<template>
  <div class="theme-selector">
    <label for="theme-select">Theme:</label>
    <select 
      id="theme-select"
      :value="currentTheme" 
      @change="handleThemeChange"
      class="theme-select"
    >
      <option 
        v-for="theme in availableThemes" 
        :key="theme.name" 
        :value="theme.name"
      >
        {{ theme.displayName }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useTheme, type Theme } from '../composables/useTheme'

const { theme, setTheme } = useTheme()

const availableThemes = [
  { name: 'light', displayName: 'Light' },
  { name: 'dark', displayName: 'Dark' },
  { name: 'auto', displayName: 'Auto' },
]

const currentTheme = theme

const handleThemeChange = async (event: Event) => {
  const target = event.target as HTMLSelectElement
  const themeName = target.value as Theme
  setTheme(themeName)
}

// Initialize theme on mount
onMounted(() => {
  // Theme is already initialized
})
</script>
