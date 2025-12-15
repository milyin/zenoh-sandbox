<template>
  <div class="section-tab-group" :class="{ 'section-collapsed': collapsed, 'section-vertical': vertical, 'is-titlebar': isTitlebar, 'boxed-mode': boxed }">
    <!-- Tab header with tabs and shared collapse button (hidden in vertical mode) -->
    <div 
      v-if="!vertical" 
      class="section-header" 
      :style="headerStyle" 
      :data-tauri-drag-region="isTitlebar ? '' : undefined"
      @dblclick="onHeaderDoubleClick"
    >
      <div class="section-tabs">
        <button
          v-for="tab in registeredTabs"
          :key="tab.id"
          class="tab-button"
          :class="{ active: activeTab === tab.id }"
          :style="getTabStyle(tab)"
          @click="selectTab(tab.id)"
          @dblclick.stop
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
    <div v-if="!collapsed" class="section-content" :style="contentStyle">
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
  backgroundColor?: string
}

// Re-export for backwards compatibility
export { SECTION_GROUP_KEY, type SectionGroupContext }
</script>

<script setup lang="ts">
import { ref, watch, provide, computed } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import CheckButton from './CheckButton.vue'

interface Props {
  activeTab?: string
  collapsed?: boolean
  vertical?: boolean
  headerBackgroundColor?: string
  isTitlebar?: boolean
  boxed?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  collapsed: false,
  vertical: false,
  isTitlebar: false,
  boxed: false
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

// Handle double-click on header to toggle window maximize (macOS zoom behavior)
async function onHeaderDoubleClick(event: MouseEvent) {
  if (!props.isTitlebar) return
  
  // Don't trigger if clicking on interactive elements
  const target = event.target as HTMLElement
  if (target.closest('button') || target.closest('.section-actions')) {
    return
  }
  
  console.log('Double-click on titlebar detected')
  
  try {
    const appWindow = getCurrentWindow()
    await appWindow.toggleMaximize()
    console.log('Toggle maximize called')
  } catch (e) {
    console.error('Failed to toggle maximize:', e)
  }
}

// Computed style for the section header - uses headerBackgroundColor prop
const headerStyle = computed(() => {
  // Header always uses headerBackgroundColor, tabs show their own section colors
  if (props.headerBackgroundColor) {
    return { backgroundColor: props.headerBackgroundColor }
  }
  return {}
})

// Get style for individual tab
function getTabStyle(tab: TabDefinition) {
  const bgColor = tab.backgroundColor || 'var(--section-default-bg, #cbd5e1)'
  
  const isActive = activeTab.value === tab.id
  if (isActive) {
    // Active tab: solid background that covers the header border
    return { 
      'background-color': bgColor,
      '--tab-bg-color': bgColor,
      'border-bottom-color': bgColor
    }
  } else {
    // Semi-transparent background for inactive tabs
    return { 
      '--tab-bg-color': bgColor,
      'background': `color-mix(in srgb, ${bgColor} 50%, transparent)`
    }
  }
}

// Computed style for content area - uses active tab's background color
const contentStyle = computed(() => {
  const activeTabData = registeredTabs.value.find(t => t.id === activeTab.value)
  if (activeTabData?.backgroundColor) {
    return { backgroundColor: activeTabData.backgroundColor }
  }
  return {}
})

// Provide context for child Sections to register themselves
provide<SectionGroupContext>(SECTION_GROUP_KEY, {
  type: 'tab',
  registerSection(id: string, title: string, icon: string, backgroundColor?: string) {
    // Only add if not already registered
    if (!registeredTabs.value.find(t => t.id === id)) {
      registeredTabs.value.push({ id, title, icon, backgroundColor })
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
  align-items: flex-end;
  background-color: var(--section-default-bg, #cbd5e1);
  border-bottom: 1px solid var(--border-color, #bbb);
  flex-shrink: 0;
  padding: var(--size-md, 4px) 0 0 0;
  gap: var(--size-xs, 2px);
}

/* Titlebar mode - header overlays native titlebar */
.is-titlebar > .section-header {
  padding-top: var(--titlebar-inset-top, 0);
  padding-left: var(--titlebar-inset-left, 0);
}

.section-tabs {
  display: flex;
  gap: var(--size-xs, 2px);
  flex: 1;
  align-items: flex-end;
  padding-left: var(--size-md, 4px);
}

.tab-button {
  display: flex;
  align-items: center;
  gap: var(--size-md, 4px);
  padding: var(--size-md, 4px) var(--size-xl, 8px);
  border: 1px solid var(--border-color, #bbb);
  border-bottom: none;
  border-radius: var(--radius-md, 4px) var(--radius-md, 4px) 0 0;
  cursor: pointer;
  font-size: var(--font-size-normal, 14px);
  font-weight: 500;
  color: var(--text-muted-color, #6c757d);
  transition: all 0.2s;
  position: relative;
  margin-bottom: -1px;
}

.tab-button:not(.active) {
  background: color-mix(in srgb, var(--tab-bg-color, var(--section-default-bg, #cbd5e1)) 50%, transparent);
}

.tab-button:hover {
  color: var(--text-color, #333);
}

.tab-button.active {
  color: var(--text-color, #333);
  font-weight: 600;
  border-bottom: 1px solid var(--tab-bg-color, var(--section-default-bg, #cbd5e1));
  z-index: 1;
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

/* Boxed mode - wraps content in a rounded border box */
.boxed-mode .section-header {
  border-bottom: none;
  background: transparent;
}

.boxed-mode .section-tabs {
  padding-left: 0;
}

.boxed-mode .section-content {
  border: 1px solid var(--border-color, #bbb);
  border-radius: 0 var(--radius-md, 4px) var(--radius-md, 4px) var(--radius-md, 4px);
  overflow: hidden;
}

.boxed-mode .tab-button.active {
  border-bottom: 1px solid var(--tab-bg-color, var(--section-default-bg, #cbd5e1));
  margin-bottom: 0;
}
</style>
