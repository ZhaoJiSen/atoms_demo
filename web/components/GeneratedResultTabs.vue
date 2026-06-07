<script setup lang="ts">
import type { GeneratedResult } from '~/types/apps'

const props = defineProps<{
  result?: GeneratedResult | null
  activeTab: string
}>()

const { t } = useI18n()

// File tree state
const selectedFile = ref<string | null>(null)
const fileContent = ref('')
const expandedFolders = ref<Set<string>>(new Set(['src']))
const fileContents = ref<Record<string, string>>({})
const activePreviewFile = ref<string | null>(null)

interface FileNode {
  name: string
  path: string
  type: 'file' | 'directory'
  children?: FileNode[]
  description?: string
  content?: string
}

function buildFileTree(files: Array<{ path: string; type: string; description?: string; content?: string }>): FileNode[] {
  const root: FileNode[] = []
  const map = new Map<string, FileNode>()

  const sorted = [...files].sort((a, b) => {
    if (a.type !== b.type) return a.type === 'directory' ? -1 : 1
    return a.path.localeCompare(b.path)
  })

  for (const file of sorted) {
    const parts = file.path.split('/').filter(Boolean)
    let currentPath = ''
    
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i]
      const parentPath = currentPath
      currentPath = currentPath ? `${currentPath}/${part}` : part
      
      if (!map.has(currentPath)) {
        const isFile = i === parts.length - 1 && file.type === 'file'
        const node: FileNode = {
          name: part,
          path: currentPath,
          type: isFile ? 'file' : 'directory',
          children: isFile ? undefined : [],
          description: isFile ? file.description : undefined,
          content: isFile ? (file as { content?: string }).content : undefined,
        }
        map.set(currentPath, node)
        
        if (parentPath) {
          const parent = map.get(parentPath)
          if (parent?.children) {
            parent.children.push(node)
          }
        } else {
          root.push(node)
        }
      }
    }
  }

  return root
}

const fileTree = computed(() => {
  if (!props.result?.fileStructure) return []
  return buildFileTree(props.result.fileStructure)
})

const allFiles = computed(() => {
  return props.result?.fileStructure.filter(file => file.type === 'file') || []
})

const expandedFolderList = computed(() => Array.from(expandedFolders.value))

const previewCode = computed(() => {
  const path = activePreviewFile.value || findDefaultPreviewFile()
  return path ? fileContents.value[path] || '' : ''
})

const previewRoutes = computed(() => {
  return (props.result?.pages || [])
    .map(page => ({
      path: normalizeRoutePath(page.path),
      name: page.name,
      filePath: routeToFilePath(page.path),
    }))
    .filter(route => Boolean(route.filePath))
})

function toggleFolder(path: string) {
  if (expandedFolders.value.has(path)) {
    expandedFolders.value.delete(path)
  } else {
    expandedFolders.value.add(path)
  }
}

function getFileContent(path: string) {
  if (Object.prototype.hasOwnProperty.call(fileContents.value, path)) {
    return fileContents.value[path]
  }

  const generatedFile = props.result?.fileStructure.find(file => file.path === path)
  return generatedFile?.content || fallbackFileContent(path)
}

function selectFile(path: string) {
  selectedFile.value = path
  fileContent.value = getFileContent(path)
  if (path.endsWith('.vue')) {
    activePreviewFile.value = path
  }
}

function normalizeRoutePath(path: string) {
  if (!path || path === 'index') return '/'
  return path.startsWith('/') ? path : `/${path}`
}

function routeToFilePath(routePath: string) {
  const route = normalizeRoutePath(routePath)
  const candidates = route === '/'
    ? ['src/pages/index.vue', 'pages/index.vue', 'src/App.vue']
    : [
        `src/pages${route}.vue`,
        `pages${route}.vue`,
        `src/pages${route}/index.vue`,
        `pages${route}/index.vue`,
      ]

  return candidates.find(path => fileContents.value[path] !== undefined)
    || candidates.find(path => allFiles.value.some(file => file.path === path))
    || ''
}

function openPreviewRoute(routePath: string) {
  const filePath = routeToFilePath(routePath)
  if (!filePath) return
  selectFile(filePath)
  activePreviewFile.value = filePath
}

function fallbackFileContent(path: string) {
  const ext = path.split('.').pop()
  const mockContents: Record<string, string> = {
    vue: `<template>
  <div class="page">
    <h1>{{ title }}</h1>
    <div class="stats">
      <div class="stat-card">
        <div class="stat-value">{{ stats.total }}</div>
        <div class="stat-label">Total</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.active }}</div>
        <div class="stat-label">Active</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.completed }}</div>
        <div class="stat-label">Completed</div>
      </div>
    </div>
    <div v-for="item in items" :key="item.id" class="card">
      <p>{{ item.name }} - {{ item.status }}</p>
    </div>
    <button class="btn" @click="addItem">Add Item</button>
  </div>
</template>`,
    ts: `export interface Config {
  name: string
  version: string
}

export function defineConfig(config: Config): Config {
  return config
}`,
    js: `export function greet(name) {
  return \`Hello, \${name}!\`
}`,
    css: `.page {
  padding: 1rem;
  max-width: 1200px;
  margin: 0 auto;
}

.stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin-bottom: 1rem;
}

.stat-card {
  background: #18181b;
  border: 1px solid #27272a;
  border-radius: 0.5rem;
  padding: 1rem;
  text-align: center;
}

.stat-value {
  font-size: 2rem;
  font-weight: 700;
}

.stat-label {
  font-size: 0.75rem;
  color: #71717a;
  text-transform: uppercase;
}`,
    json: `{
  "name": "my-app",
  "version": "1.0.0"
}`,
  }
  return mockContents[ext || ''] || `// Content of ${path}`
}

const fileLanguage = computed(() => {
  if (!selectedFile.value) return 'text'
  const ext = selectedFile.value.split('.').pop()
  const langMap: Record<string, string> = {
    vue: 'html',
    ts: 'javascript',
    js: 'javascript',
    css: 'css',
    json: 'javascript',
  }
  return langMap[ext || ''] || 'text'
})

function onCodeChange(code: string) {
  fileContent.value = code
  if (selectedFile.value) {
    fileContents.value = {
      ...fileContents.value,
      [selectedFile.value]: code,
    }
  }
  if (selectedFile.value?.endsWith('.vue')) {
    activePreviewFile.value = selectedFile.value
  }
}

function findDefaultPreviewFile() {
  const vueFiles = allFiles.value
    .map(file => file.path)
    .filter(path => path.endsWith('.vue'))

  return vueFiles.find(path => path === 'src/pages/index.vue')
    || vueFiles.find(path => path === 'pages/index.vue')
    || vueFiles.find(path => path === 'src/App.vue')
    || vueFiles[0]
    || null
}

function collectDirectoryPaths(nodes: FileNode[]) {
  const paths: string[] = []
  const visit = (node: FileNode) => {
    if (node.type === 'directory') {
      paths.push(node.path)
      node.children?.forEach(visit)
    }
  }
  nodes.forEach(visit)
  return paths
}

watch(
  () => props.result?.fileStructure,
  () => {
    const nextContents: Record<string, string> = {}
    for (const file of allFiles.value) {
      nextContents[file.path] = file.content || fallbackFileContent(file.path)
    }
    fileContents.value = nextContents

    const nextPreviewFile = findDefaultPreviewFile()
    activePreviewFile.value = nextPreviewFile
    if (nextPreviewFile) {
      selectFile(nextPreviewFile)
    } else {
      selectedFile.value = null
      fileContent.value = ''
    }

    expandedFolders.value = new Set(collectDirectoryPaths(fileTree.value))
  },
  { immediate: true },
)

onMounted(() => {
  window.addEventListener('message', handlePreviewMessage)
})

onBeforeUnmount(() => {
  window.removeEventListener('message', handlePreviewMessage)
})

function handlePreviewMessage(event: MessageEvent) {
  if (!event.data || event.data.type !== 'atoms-preview-route') return
  openPreviewRoute(String(event.data.route || '/'))
}
</script>

<template>
  <div v-if="!result" class="text-center py-8">
    <FileText class="w-8 h-8 text-zinc-600 mx-auto mb-2" />
    <p class="text-sm text-zinc-500">{{ t('common.notGenerated') }}</p>
  </div>

  <div v-else>
    <!-- Preview with Sandbox -->
    <div v-if="activeTab === 'preview'" class="h-[70vh] flex gap-4">
      <div class="flex-1 rounded-lg border border-zinc-800 overflow-hidden bg-zinc-950 flex flex-col">
        <div class="px-4 py-2 bg-zinc-900 border-b border-zinc-800 flex items-center gap-2 flex-shrink-0">
          <Eye class="w-4 h-4 text-zinc-500" />
          <span class="text-xs font-medium text-zinc-400">Preview</span>
          <span v-if="activePreviewFile" class="ml-auto text-xs font-mono text-zinc-500 truncate">
            {{ activePreviewFile }}
          </span>
          <UBadge v-if="previewCode" color="green" variant="subtle" size="xs">Live</UBadge>
        </div>
        <div v-if="previewRoutes.length > 1" class="px-4 py-2 bg-zinc-950 border-b border-zinc-800 flex items-center gap-2 overflow-x-auto flex-shrink-0">
          <button
            v-for="route in previewRoutes"
            :key="route.path"
            @click="openPreviewRoute(route.path)"
            :class="[
              'px-2 py-1 rounded text-xs whitespace-nowrap transition-colors',
              activePreviewFile === route.filePath
                ? 'bg-violet-500/20 text-violet-300'
                : 'text-zinc-500 hover:text-zinc-300 hover:bg-zinc-800'
            ]"
          >
            {{ route.name || route.path }}
          </button>
        </div>
        <div class="flex-1 min-h-0 overflow-auto">
          <SandboxPreview
            v-if="previewCode"
            :code="previewCode"
            :file-name="activePreviewFile || undefined"
            :routes="previewRoutes"
          />
          <div v-else class="flex items-center justify-center h-full text-zinc-600">
            <div class="text-center">
              <Eye class="w-12 h-12 mx-auto mb-2" />
              <p class="text-sm">{{ t('workspace.previewHint') }}</p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Product Spec -->
    <div v-else-if="activeTab === 'spec'" class="space-y-6">
      <table class="w-full text-sm">
        <tbody class="divide-y divide-zinc-800">
          <tr>
            <td class="py-3 pr-4 text-zinc-500 w-32">{{ t('common.name') }}</td>
            <td class="py-3 text-zinc-200">{{ result.productSpec.name }}</td>
          </tr>
          <tr>
            <td class="py-3 pr-4 text-zinc-500">{{ t('common.description') }}</td>
            <td class="py-3 text-zinc-300">{{ result.productSpec.description }}</td>
          </tr>
          <tr>
            <td class="py-3 pr-4 text-zinc-500 align-top">{{ t('common.targetUsers') }}</td>
            <td class="py-3">
              <div class="flex flex-wrap gap-1.5">
                <UBadge v-for="user in result.productSpec.targetUsers" :key="user" color="gray" variant="subtle" size="xs">
                  {{ user }}
                </UBadge>
              </div>
            </td>
          </tr>
          <tr>
            <td class="py-3 pr-4 text-zinc-500 align-top">{{ t('common.coreFeatures') }}</td>
            <td class="py-3">
              <div class="flex flex-wrap gap-1.5">
                <UBadge v-for="feature in result.productSpec.coreFeatures" :key="feature" color="primary" variant="subtle" size="xs">
                  {{ feature }}
                </UBadge>
              </div>
            </td>
          </tr>
          <tr>
            <td class="py-3 pr-4 text-zinc-500 align-top">{{ t('common.userFlow') }}</td>
            <td class="py-3">
              <div class="flex flex-wrap gap-2">
                <div v-for="(step, index) in result.productSpec.userFlow" :key="step" class="flex items-center gap-1">
                  <span class="w-5 h-5 rounded-full bg-zinc-800 flex items-center justify-center text-xs text-zinc-500">
                    {{ index + 1 }}
                  </span>
                  <span class="text-sm text-zinc-300">{{ step }}</span>
                </div>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Pages -->
    <div v-else-if="activeTab === 'pages'" class="space-y-3">
      <div
        v-for="page in result.pages"
        :key="page.path"
        class="p-3 rounded-lg border border-zinc-800"
      >
        <div class="flex items-center gap-2 mb-1">
          <File class="w-3.5 h-3.5 text-zinc-500" />
          <span class="text-sm font-medium font-mono">{{ page.path }}</span>
        </div>
        <p class="text-xs text-zinc-500 mb-2">{{ page.description }}</p>
        <div class="flex flex-wrap gap-1">
          <UBadge v-for="comp in page.components" :key="comp" color="gray" variant="subtle" size="xs">
            {{ comp }}
          </UBadge>
        </div>
      </div>
      <div v-if="result.pages.length === 0" class="text-xs text-zinc-600 text-center py-4">
        {{ t('common.notGenerated') }}
      </div>
    </div>

    <!-- API -->
    <div v-else-if="activeTab === 'api'" class="space-y-2">
      <div
        v-for="api in result.apis"
        :key="api.path"
        class="flex items-center gap-3 p-3 rounded-lg border border-zinc-800 hover:border-zinc-700 transition-colors"
      >
        <UBadge
          :color="(api.method === 'GET' ? 'green' : api.method === 'POST' ? 'primary' : api.method === 'DELETE' ? 'red' : 'gray') as any"
          variant="solid"
          size="xs"
          class="w-16 justify-center font-mono"
        >
          {{ api.method }}
        </UBadge>
        <span class="text-sm font-mono flex-1 text-zinc-300">{{ api.path }}</span>
        <span class="text-xs text-zinc-500">{{ api.description }}</span>
      </div>
      <div v-if="result.apis.length === 0" class="text-xs text-zinc-600 text-center py-4">
        {{ t('common.notGenerated') }}
      </div>
    </div>

    <!-- Data Models -->
    <div v-else-if="activeTab === 'models'" class="space-y-6">
      <div v-for="model in result.dataModels" :key="model.name">
        <div class="flex items-center gap-2 mb-3">
          <span class="text-sm font-medium font-mono text-zinc-200">{{ model.name }}</span>
          <span class="text-xs text-zinc-500">- {{ model.description }}</span>
        </div>
        <table class="w-full text-sm border border-zinc-800 rounded-lg overflow-hidden">
          <thead>
            <tr class="bg-zinc-800/50 text-zinc-500 text-xs">
              <th class="text-left px-3 py-2 font-medium">{{ t('common.field') }}</th>
              <th class="text-left px-3 py-2 font-medium">{{ t('common.type') }}</th>
              <th class="text-left px-3 py-2 font-medium">{{ t('common.required') }}</th>
              <th class="text-left px-3 py-2 font-medium">{{ t('common.description') }}</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-zinc-800">
            <tr v-for="field in model.fields" :key="field.name">
              <td class="px-3 py-2 font-mono text-violet-400">{{ field.name }}</td>
              <td class="px-3 py-2 text-zinc-400 font-mono">{{ field.type }}</td>
              <td class="px-3 py-2">
                <UBadge v-if="field.required" color="red" variant="subtle" size="xs">{{ t('common.required') }}</UBadge>
                <span v-else class="text-zinc-600">-</span>
              </td>
              <td class="px-3 py-2 text-zinc-400">{{ field.description }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <div v-if="result.dataModels.length === 0" class="text-xs text-zinc-600 text-center py-4">
        {{ t('common.notGenerated') }}
      </div>
    </div>

    <!-- File Structure with Tree + CodeMirror -->
    <div v-else-if="activeTab === 'files'" class="flex gap-4 h-[60vh]">
      <!-- File Tree -->
      <div class="w-64 flex-shrink-0 overflow-y-auto border-r border-zinc-800 pr-4">
        <div class="text-xs text-zinc-500 mb-3 font-medium uppercase tracking-wider">Files</div>
        <div class="space-y-0.5">
          <FileTreeNode
            v-for="node in fileTree"
            :key="node.path"
            :node="node"
            :selected-file="selectedFile"
            :expanded-paths="expandedFolderList"
            @select="selectFile"
            @toggle="toggleFolder"
          />
        </div>
      </div>

      <!-- Code Editor -->
      <div class="flex-1 overflow-hidden rounded-lg border border-zinc-800">
        <div v-if="selectedFile" class="h-full flex flex-col">
          <div class="flex items-center justify-between px-3 py-2 bg-zinc-900 border-b border-zinc-800">
            <span class="text-xs font-mono text-zinc-400">{{ selectedFile }}</span>
            <div class="flex items-center gap-2">
              <UBadge color="gray" variant="subtle" size="xs">{{ fileLanguage }}</UBadge>
              <UButton v-if="selectedFile?.endsWith('.vue')" size="xs" variant="ghost" color="primary" @click="activePreviewFile = selectedFile">
                <Play class="w-3 h-3 mr-1" />
                Run
              </UButton>
            </div>
          </div>
          <div class="flex-1 overflow-auto">
            <CodeEditor
              :model-value="fileContent"
              :language="fileLanguage"
              :read-only="false"
              @update:model-value="onCodeChange"
            />
          </div>
        </div>
        <div v-else class="flex items-center justify-center h-full text-zinc-600">
          <div class="text-center">
            <FileCode class="w-12 h-12 mx-auto mb-2" />
            <p class="text-sm">Select a file to edit</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="activeTab === 'files' && result.fileStructure.length === 0" class="text-xs text-zinc-600 text-center py-4">
      {{ t('common.notGenerated') }}
    </div>
  </div>
</template>
