<template>
  <NumberOptionSelect
    :model-value="modelValue"
    :disabled="disabled ?? false"
    :options="options"
    :label="label ?? 'Allowed Destination'"
     @update:model-value="$emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import type { Locality } from '../composables/zenohTypes'
import type { OptionItem } from '../composables/zenohDemo/safeUtils'
import NumberOptionSelect from './NumberOptionSelect.vue'

interface Props {
  modelValue: Locality | undefined
  disabled?: boolean
  options: OptionItem[]
  label?: string
}

interface Emits {
  (e: 'update:modelValue', value: Locality | undefined): void
}

defineProps<Props>()
defineEmits<Emits>()
</script>
