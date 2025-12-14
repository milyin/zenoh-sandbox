<template>
  <div 
    v-show="isVisible"
    class="entity-group" 
    :class="[sectionClass, { disabled: disabled, 'section-collapsed': collapsed }]"
  >
    <!-- Standard header (shown in standalone mode) -->
    <div v-if="!isInTabGroup" class="section-header">
      <div class="section-icon">{{ icon }}</div>
      <div class="section-title">{{ title }}</div>

      <!-- Section actions - separate but sharing horizontal space -->
      <div class="section-actions">
        <slot name="actions" />
        <!-- Fold/Unfold button (always present - section is always collapsible) -->
        <CheckButton
          :pressed="collapsed"
          @update:pressed="emit('update:collapsed', $event)"
        />
      </div>
    </div>

    <!-- Actions shown as controls bar when in tab group mode -->
    <div v-if="isInTabGroup && $slots['actions'] && !collapsed" class="section-controls">
      <slot name="actions" />
    </div>

    <div v-if="!collapsed" class="section-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.section-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.section-controls {
  display: flex;
  align-items: center;
  gap: var(--size-lg, 6px);
  padding: var(--size-lg, 6px) var(--size-xl, 8px);
  background: var(--section-controls-bg, #f0f0f0);
  border-bottom: 1px solid var(--border-color, #ddd);
  flex-wrap: wrap;
}

.section-collapsed .section-header {
  border-bottom: none;
}
</style>

<script setup lang="ts">
import { inject, onMounted, onUnmounted, computed } from 'vue'
import CheckButton from './CheckButton.vue'
import { SECTION_TAB_GROUP_KEY, type SectionTabGroupContext } from './SectionTabGroup.vue'

interface Props {
  id?: string
  title: string
  icon: string
  sectionClass?: string
  disabled?: boolean
  collapsed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  sectionClass: '',
  disabled: false,
  collapsed: false
})

const emit = defineEmits<{
  (e: 'update:collapsed', value: boolean): void
}>()

// If inside a SectionTabGroup, register this section
const tabGroupContext = inject<SectionTabGroupContext | null>(SECTION_TAB_GROUP_KEY, null)

// Check if we're inside a tab group
const isInTabGroup = computed(() => !!tabGroupContext && !!props.id)

// Determine visibility - always visible if standalone, or if active tab in group
const isVisible = computed(() => {
  if (!tabGroupContext || !props.id) return true
  return tabGroupContext.isActiveTab(props.id)
})

onMounted(() => {
  if (tabGroupContext && props.id) {
    tabGroupContext.registerSection(props.id, props.title, props.icon)
  }
})

onUnmounted(() => {
  if (tabGroupContext && props.id) {
    tabGroupContext.unregisterSection(props.id)
  }
})
</script>
