<template>
  <div class="section-tab-group" :class="{ 'section-collapsed': collapsed, 'section-vertical': vertical }">
    <!-- Tab header with tabs and shared collapse button (hidden in vertical mode) -->
    <div v-if="!vertical" class="section-header">
      <div class="section-tabs">
        <button
          v-for="tab in registeredTabs"
          :key="tab.id"
          class="tab-button"
          :class="{ active: activeTab === tab.id }"
          @click="selectTab(tab.id)"
        >
          <span class="tab-icon">{{ tab.icon }}</span>
          <span class="tab-title">{{ tab.title }}</span>
        </button>
      </div>
      <div class="section-actions">
        <slot name="actions" />
        <!-- Shared collapse button for all tabs -->
        <CheckButton
          :pressed="collapsed"
          @update:pressed="emit('update:collapsed', $event)"
        />
      </div>
    </div>

    <!-- Tab content area (only shown when not collapsed) -->
    <div v-if="!collapsed" class="section-content">
      <!-- Default slot renders all Section children directly -->
      <slot />
    </div>
  </div>
</template>

<script lang="ts">
import { SECTION_GROUP_KEY, type SectionGroupContext } from './sectionGroupContext'

export interface TabDefinition {
  id: string
  title: string
  icon: string
}

// Re-export for backwards compatibility
export { SECTION_GROUP_KEY, type SectionGroupContext }
</script>

<script setup lang="ts">
import { ref, watch, provide } from 'vue'
import CheckButton from './CheckButton.vue'

interface Props {
  activeTab?: string
  collapsed?: boolean
  vertical?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  collapsed: false,
  vertical: false
})

const emit = defineEmits<{
  (e: 'update:collapsed', value: boolean): void
  (e: 'update:activeTab', value: string): void
}>()

// Sections register themselves here - array preserves registration order
const registeredTabs = ref<TabDefinition[]>([])

// Track active tab internally, sync with prop
const activeTab = ref(props.activeTab || '')

// Set initial active tab when first section registers
watch(registeredTabs, (tabs) => {
  if (tabs.length > 0 && !activeTab.value) {
    activeTab.value = tabs[0].id
  }
}, { deep: true })

// Watch for external activeTab prop changes
watch(() => props.activeTab, (newVal) => {
  if (newVal && newVal !== activeTab.value) {
    activeTab.value = newVal
  }
})

function selectTab(tabId: string) {
  activeTab.value = tabId
  emit('update:activeTab', tabId)
}

// Provide context for child Sections to register themselves
provide<SectionGroupContext>(SECTION_GROUP_KEY, {
  type: 'tab',
  registerSection(id: string, title: string, icon: string) {
    // Only add if not already registered
    if (!registeredTabs.value.find(t => t.id === id)) {
      registeredTabs.value.push({ id, title, icon })
    }
  },
  unregisterSection(id: string) {
    const index = registeredTabs.value.findIndex(t => t.id === id)
    if (index !== -1) {
      registeredTabs.value.splice(index, 1)
    }
  },
  isSectionVisible(id: string) {
    return activeTab.value === id
  }
})
</script>

<style scoped>
.section-tab-group {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.section-header {
  display: flex;
  align-items: center;
  background-color: var(--section-default-bg, #cbd5e1);
  border-bottom: 1px solid var(--border-color, #ddd);
  flex-shrink: 0;
  padding: var(--size-lg, 6px) 0;
}

.section-tabs {
  display: flex;
  gap: 0;
  flex: 1;
}

.tab-button {
  display: flex;
  align-items: center;
  gap: var(--size-md, 4px);
  padding: 0 var(--size-xl, 8px);
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: var(--font-size-normal, 14px);
  font-weight: 500;
  color: var(--text-muted-color, #6c757d);
  transition: all 0.2s;
}

.tab-button:hover {
  background: var(--tab-hover-bg, rgba(0, 0, 0, 0.05));
  color: var(--text-color, #333);
}

.tab-button.active {
  color: var(--primary-color, #007bff);
  background: var(--tab-active-bg, rgba(255, 255, 255, 0.5));
  font-weight: 600;
}

.tab-icon {
  font-size: var(--font-size-normal, 14px);
}

.tab-title {
  white-space: nowrap;
}

.section-actions {
  display: flex;
  align-items: center;
  gap: var(--size-md, 4px);
  padding: 0 var(--size-xl, 8px);
}

.section-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.section-collapsed .section-header {
  border-bottom: none;
}
</style>
