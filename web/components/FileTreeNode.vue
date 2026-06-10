<script setup lang="ts">
interface FileTreeNode {
  name: string
  path: string
  type: 'file' | 'directory'
  children?: FileTreeNode[]
}

const props = defineProps<{
  node: FileTreeNode
  selectedFile?: string | null
  expandedPaths: string[]
  streamingPaths?: string[]
  depth?: number
}>()

const emit = defineEmits<{
  select: [path: string]
  toggle: [path: string]
}>()

const isExpanded = computed(() => props.expandedPaths.includes(props.node.path))
const isWriting = computed(() =>
  props.node.type === 'file' && (props.streamingPaths || []).includes(props.node.path),
)
const paddingLeft = computed(() => `${(props.depth || 0) * 14}px`)

function handleClick() {
  if (props.node.type === 'directory') {
    emit('toggle', props.node.path)
    return
  }

  emit('select', props.node.path)
}
</script>

<template>
  <div>
    <UButton
      @click="handleClick"
      :variant="selectedFile === node.path ? 'soft' : 'ghost'"
      :color="selectedFile === node.path ? 'primary' : 'gray'"
      size="xs"
      block
      class="justify-start"
      :style="{ paddingLeft }"
    >
      <ChevronRight
        v-if="node.type === 'directory'"
        :class="['w-3 h-3 transition-transform', isExpanded ? 'rotate-90' : '']"
      />
      <span v-else class="w-3" />
      <Folder v-if="node.type === 'directory'" class="w-4 h-4 text-violet-400 mr-1" />
      <File v-else class="w-4 h-4 text-zinc-500 mr-1" />
      <span class="font-mono truncate">{{ node.name }}</span>
      <Loader2 v-if="isWriting" class="w-3 h-3 ml-auto text-violet-400 animate-spin flex-shrink-0" />
    </UButton>

    <div v-if="node.type === 'directory' && isExpanded">
      <FileTreeNode
        v-for="child in node.children || []"
        :key="child.path"
        :node="child"
        :selected-file="selectedFile"
        :expanded-paths="expandedPaths"
        :streaming-paths="streamingPaths"
        :depth="(depth || 0) + 1"
        @select="emit('select', $event)"
        @toggle="emit('toggle', $event)"
      />
    </div>
  </div>
</template>
