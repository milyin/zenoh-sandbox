<template>
  <div class="entity">
    <!-- Wrapper to make header and actions share horizontal space -->
    <div class="entity-header">
      <div class="entity-title" :class="{ 'entity-title-link': titleLink }" @click="handleTitleClick">
        {{ title }}
      </div>
      <div v-if="session" class="entity-session" :class="{ 'selected': isSessionSelected }">
        {{ session }}
      </div>
      <div v-if=" descr" class="entity-descr">
        {{ descr }}
      </div>
      <!-- Entity actions - separate but sharing horizontal space -->
      <div class="entity-actions">
        <CheckButton
          v-if="$slots['info']"
          v-model:pressed="infoExpanded"
          label="Info..."
        />
        <CheckButton
          v-if="$slots['edits']"
          v-model:pressed="editsExpanded"
          label="Edit..."
        />
        <slot name="actions" />
      </div>
    </div>

    <!-- Entity edits - now at same level as entity-header -->
    <div v-if="editsExpanded && $slots['edits']" class="entity-edits">
      <slot name="edits" />
    </div>

    <!-- Default slot for general content -->
    <slot />

    <!-- Info section - now at same level as others -->
    <div v-if="$slots['info'] && infoExpanded" class="entity-info">
      <slot name="info" />
    </div>

    <!-- Special slot for sub-entities -->
    <div v-if="$slots['sub-entities']" class="sub-entities">
      <slot name="sub-entities" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import CheckButton from './CheckButton.vue'

interface Props {
  title: string;
  descr?: string;
  session?: string | null;
  selectedSession?: string | null;
  titleLink?: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'title-click': []
}>();

const editsExpanded = defineModel<boolean>("editsExpanded", {
  default: false,
});

const infoExpanded = ref(false);

const isSessionSelected = computed(() => {
  return props.session && props.selectedSession === props.session;
});

const handleTitleClick = () => {
  if (props.titleLink) {
    emit('title-click');
  }
};
</script>

<style scoped>
.entity-descr {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.entity-title-link {
  cursor: pointer;
  color: var(--primary-color, #007bff);
  text-decoration: underline;
}

.entity-title-link:hover {
  color: var(--primary-hover-color, #0056b3);
}
</style>
