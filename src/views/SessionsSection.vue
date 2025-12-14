<template>
  <Section
    id="sessions"
    title="Sessions"
    icon="🔌"
    background-color="var(--section-session-bg, #bae6fd)"
    :collapsed="collapsed"
    @update:collapsed="emit('update:collapsed', $event)"
  >
    <SessionsTab />
  </Section>
</template>

<script setup lang="ts">
import Section from '../components/Section.vue';
import SessionsTab from '../SessionsTab.vue';

interface Props {
  collapsed?: boolean;
}

withDefaults(defineProps<Props>(), {
  collapsed: false
});

const emit = defineEmits<{
  (e: 'update:collapsed', value: boolean): void
}>();
</script>
