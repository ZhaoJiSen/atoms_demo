<script setup lang="ts">
import { Codemirror } from 'vue-codemirror'
import { javascript } from '@codemirror/lang-javascript'
import { html } from '@codemirror/lang-html'
import { css } from '@codemirror/lang-css'
import { oneDark } from '@codemirror/theme-one-dark'

const props = defineProps<{
  modelValue: string
  language?: string
  readOnly?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const extensions = computed(() => {
  const exts = [oneDark]
  
  switch (props.language) {
    case 'javascript':
    case 'js':
    case 'ts':
    case 'typescript':
      exts.push(javascript())
      break
    case 'html':
    case 'vue':
      exts.push(html())
      break
    case 'css':
      exts.push(css())
      break
  }
  
  return exts
})

const onChange = (value: string) => {
  emit('update:modelValue', value)
}
</script>

<template>
  <Codemirror
    :model-value="modelValue"
    :extensions="extensions"
    :disabled="readOnly"
    :style="{ height: '100%', minHeight: '300px' }"
    @update:model-value="onChange"
  />
</template>
