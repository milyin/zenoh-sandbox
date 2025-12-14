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

    <div v-if="!collapsed" class="section-content" :style="contentStyle">
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
import { SECTION_GROUP_KEY, type SectionGroupContext } from './sectionGroupContext'

interface Props {
  id?: string
  title: string
  icon: string
  sectionClass?: string
  backgroundColor?: string
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

// Inject the group context (either tab group or list group)
const groupContext = inject<SectionGroupContext | null>(SECTION_GROUP_KEY, null)

// Check if we're inside a tab group (with an id)
const isInTabGroup = computed(() => groupContext?.type === 'tab' && !!props.id)

// Determine visibility based on group context
const isVisible = computed(() => {
  if (!groupContext || !props.id) return true
  return groupContext.isSectionVisible(props.id)
})

// Computed style for section content background
const contentStyle = computed(() => {
  if (props.backgroundColor) {
    return { backgroundColor: props.backgroundColor }
  }
  return {}
})

onMounted(() => {
  if (groupContext && props.id) {
    groupContext.registerSection(props.id, props.title, props.icon, props.backgroundColor)
  }
})

onUnmounted(() => {
  if (groupContext && props.id) {
    groupContext.unregisterSection(props.id)
  }
})
</script>
