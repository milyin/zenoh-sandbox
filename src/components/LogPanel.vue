<template>
  <Section :title="title || 'Logs'" :icon="icon || '📜'" section-class="log-panel-section">
    <template #actions>
      <!-- Filter selector (if filterOptions provided) -->
      <select v-if="filterOptions" v-model="currentFilter" class="log-filter">
        <option v-for="opt in filterOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>

      <!-- Auto-scroll toggle -->
      <label class="auto-scroll-toggle">
        <input type="checkbox" v-model="autoScrollEnabled" />
        Auto-scroll
      </label>

      <!-- Clear button (if enabled) -->
      <button v-if="showClearButton" @click="handleClear">
        Clear
      </button>
    </template>

    <div class="log-panel-content">
      <div ref="logContainer" class="log-entries">
        <!-- Dynamic log entry rendering -->
        <div
          v-for="(entry, index) in visibleLogs"
          :key="`${entry.timestamp}-${index}`"
          class="log-entry"
          :class="getEntryClass(entry)"
        >
          <!-- Timestamp (always shown) -->
          <span class="log-timestamp">{{ formatTime(entry.timestamp) }}</span>

          <!-- Level (shown if present) -->
          <span v-if="entry.level" class="log-level">{{ entry.level }}</span>

          <!-- Type (shown if present) -->
          <span v-if="entry.type" class="log-type">[{{ entry.type.toUpperCase() }}]</span>

          <!-- Target (shown if present) -->
          <span v-if="entry.target" class="log-target">{{ entry.target }}</span>

          <!-- Message (always shown) -->
          <span class="log-message">{{ entry.message }}</span>

          <!-- Data (shown if present) -->
          <ParameterDisplay
            v-if="entry.data"
            :type="entry.type || 'neutral'"
            :data="entry.data"
          />
        </div>

        <!-- Load more button (if pagination callback provided and more logs available) -->
        <button
          v-if="onLoadMore && hasMoreLogs"
          @click="handleLoadMore"
          :disabled="isLoadingMore"
          class="load-more-btn"
        >
          {{ isLoadingMore ? 'Loading...' : 'Load More' }}
        </button>

        <!-- Empty state -->
        <div v-if="visibleLogs.length === 0" class="empty-logs">
          No logs available
        </div>
      </div>
    </div>
  </Section>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import Section from './Section.vue';
import ParameterDisplay from './ParameterDisplay.vue';

interface LogEntry {
  timestamp: string | Date;
  message: string;
  level?: string;
  type?: "info" | "success" | "error" | "data";
  target?: string;
  data?: Record<string, any>;
}

interface FilterOption {
  value: string;
  label: string;
}

interface Props {
  title?: string;
  icon?: string;
  logs?: LogEntry[];
  eventName?: string;
  filterOptions?: FilterOption[];
  selectedFilter?: string | null;
  onLoadMore?: (currentCount: number) => Promise<LogEntry[]>;
  onClear?: () => void;
  showClearButton?: boolean;
  autoScroll?: boolean;
  maxEntries?: number;
}

const props = withDefaults(defineProps<Props>(), {
  showClearButton: true,
  autoScroll: false,
  maxEntries: 500
});

const emit = defineEmits<{
  'update:selectedFilter': [value: string | null];
}>();

// Local state
const currentFilter = computed({
  get: () => props.selectedFilter,
  set: (value) => emit('update:selectedFilter', value)
});

const internalLogsCache = ref<Map<string, LogEntry[]>>(new Map());
const autoScrollEnabled = ref(props.autoScroll);
const isLoadingMore = ref(false);
const hasMoreLogs = ref(true);
const logContainer = ref<HTMLElement | null>(null);
const eventUnlisteners = ref<UnlistenFn[]>([]);

// Computed visible logs
const visibleLogs = computed(() => {
  // If logs prop provided (direct array), use it
  if (props.logs) return props.logs;

  // If eventName provided, use internal cache for that event
  if (props.eventName && props.selectedFilter) {
    return internalLogsCache.value.get(props.selectedFilter) || [];
  }

  return [];
});

// Methods
const formatTime = (timestamp: string | Date): string => {
  const date = typeof timestamp === 'string' ? new Date(timestamp) : timestamp;
  return date.toLocaleTimeString();
};

const getEntryClass = (entry: LogEntry): string => {
  const classes: string[] = [];

  if (entry.level) {
    classes.push(`log-${entry.level.toLowerCase()}`);
  }

  if (entry.type) {
    classes.push(`log-type-${entry.type}`);
  }

  return classes.join(' ');
};

const handleLoadMore = async () => {
  if (!props.onLoadMore || isLoadingMore.value) return;

  isLoadingMore.value = true;
  try {
    const newLogs = await props.onLoadMore(visibleLogs.value.length);
    if (newLogs.length === 0) {
      hasMoreLogs.value = false;
    }
    // Logs are appended by parent component
  } catch (error) {
    console.error('Failed to load more logs:', error);
  } finally {
    isLoadingMore.value = false;
  }
};

const handleClear = () => {
  if (props.onClear) {
    props.onClear();
  } else {
    // Default: clear internal cache
    internalLogsCache.value.clear();
  }
};

const preserveScrollPosition = (newLogsCount: number) => {
  if (!logContainer.value) return;
  const scrollBefore = logContainer.value.scrollTop;

  nextTick(() => {
    if (logContainer.value) {
      // Estimate height per log entry (adjust based on actual styling)
      const estimatedHeight = newLogsCount * 30;
      logContainer.value.scrollTop = scrollBefore + estimatedHeight;
    }
  });
};

// Event listening setup (only if eventName provided)
onMounted(async () => {
  if (!props.eventName) return;

  const unlisten = await listen(props.eventName, (event: any) => {
    const { zid, entries } = event.payload;

    // Filter by selected filter if provided
    if (props.selectedFilter && zid !== props.selectedFilter) return;

    const existing = internalLogsCache.value.get(zid) || [];
    const updated = [...entries, ...existing];

    // Trim to maxEntries
    if (updated.length > props.maxEntries) {
      updated.splice(props.maxEntries);
    }

    internalLogsCache.value.set(zid, updated);

    // Scroll handling
    if (!autoScrollEnabled.value) {
      preserveScrollPosition(entries.length);
    } else {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = 0;
        }
      });
    }
  });

  eventUnlisteners.value.push(unlisten);
});

// Auto-scroll when logs change (for direct logs prop)
watch(
  () => props.logs,
  () => {
    if (autoScrollEnabled.value && logContainer.value) {
      nextTick(() => {
        if (logContainer.value) {
          logContainer.value.scrollTop = 0;
        }
      });
    }
  },
  { deep: true }
);

// Cleanup
onUnmounted(() => {
  eventUnlisteners.value.forEach(unlisten => unlisten());
});
</script>

<style scoped>
.log-panel-section {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.log-panel-content {
  flex: 1;
  overflow: hidden;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.log-entries {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  font-family: monospace;
  font-size: 0.85rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  padding: var(--spacing-content);
}

.log-entry {
  display: grid;
  grid-template-columns: auto auto 1fr 2fr;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  border-left: 3px solid transparent;
  font-size: 0.8rem;
  word-break: break-word;
}

.log-entry:hover {
  background: var(--hover-bg-color, #f0f0f0);
}

.log-timestamp {
  color: var(--log-neutral-color, #666);
  white-space: nowrap;
}

.log-level {
  font-weight: bold;
  white-space: nowrap;
}

.log-type {
  font-weight: bold;
  white-space: nowrap;
}

.log-target {
  color: var(--log-neutral-color, #666);
  overflow: hidden;
  text-overflow: ellipsis;
}

.log-message {
  word-break: break-word;
}

/* Log level colors (for runtime logs) */
.log-trace {
  border-left-color: #999;
}
.log-trace .log-level {
  color: #999;
}

.log-debug {
  border-left-color: #0d6efd;
}
.log-debug .log-level {
  color: #0d6efd;
}

.log-info {
  border-left-color: #198754;
}
.log-info .log-level {
  color: #198754;
}

.log-warn {
  border-left-color: #ffc107;
}
.log-warn .log-level {
  color: #ffc107;
}

.log-error {
  border-left-color: #dc3545;
}
.log-error .log-level {
  color: #dc3545;
}

/* Log type colors (for activity logs) */
.log-type-info {
  border-left-color: var(--log-info-color, #0d6efd);
}
.log-type-info .log-type {
  color: var(--log-info-color, #0d6efd);
}

.log-type-success {
  border-left-color: var(--log-success-color, #198754);
}
.log-type-success .log-type {
  color: var(--log-success-color, #198754);
}

.log-type-error {
  border-left-color: var(--log-error-color, #dc3545);
}
.log-type-error .log-type {
  color: var(--log-error-color, #dc3545);
}

.log-type-data {
  border-left-color: var(--log-data-color, #6c757d);
}
.log-type-data .log-type {
  color: var(--log-data-color, #6c757d);
}

.empty-logs {
  text-align: center;
  color: var(--log-neutral-color, #666);
  padding: 2rem;
}

.load-more-btn {
  margin: 1rem auto;
  padding: 0.5rem 1rem;
}

.log-filter {
  margin-right: 0.5rem;
}

.auto-scroll-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  margin-right: 0.5rem;
  cursor: pointer;
}

.auto-scroll-toggle input[type="checkbox"] {
  cursor: pointer;
}
</style>
