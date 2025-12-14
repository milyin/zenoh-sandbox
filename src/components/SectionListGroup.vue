<template>
  <div class="section-list-group">
    <slot />
  </div>
</template>

<script lang="ts">
import { SECTION_GROUP_KEY, type SectionGroupContext } from './sectionGroupContext'

// Re-export for convenience
export { SECTION_GROUP_KEY, type SectionGroupContext }
</script>

<script setup lang="ts">
import { provide } from 'vue'

// Provide context to child Sections - list groups show all sections
provide<SectionGroupContext>(SECTION_GROUP_KEY, {
  type: 'list',
  registerSection(_id: string, _title: string, _icon: string, _backgroundColor?: string) {
    // List groups don't need to track sections
  },
  unregisterSection(_id: string) {
    // List groups don't need to track sections
  },
  isSectionVisible(_id: string) {
    // All sections are always visible in a list group
    return true
  }
})
</script>

<style scoped>
.section-list-group {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}
</style>
