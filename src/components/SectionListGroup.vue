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
  width: 100%;
  gap: var(--size-lg, 6px);
  padding: var(--size-lg, 6px);
  background: transparent;
}

/* Style direct Section children with borders and rounded corners */
.section-list-group :deep(> .entity-group) {
  border: 1px solid var(--border-color, #ddd);
  border-radius: var(--radius-lg, 6px);
  overflow: hidden;
}

.section-list-group :deep(> .entity-group > .section-header) {
  border-radius: var(--radius-lg, 6px) var(--radius-lg, 6px) 0 0;
}

.section-list-group :deep(> .entity-group.section-collapsed > .section-header) {
  border-radius: var(--radius-lg, 6px);
}
</style>
