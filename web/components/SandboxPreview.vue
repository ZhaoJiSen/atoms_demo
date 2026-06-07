<script setup lang="ts">
const props = defineProps<{
  code: string
  fileName?: string
  routes?: Array<{ path: string; filePath: string; name: string }>
}>()

const error = ref<string | null>(null)
const srcdoc = ref('')

function extractBlock(code: string, tag: string) {
  const match = code.match(new RegExp(`<${tag}[^>]*>([\\s\\S]*?)<\\/${tag}>`, 'i'))
  return match?.[1]?.trim() || ''
}

function extractDataValues(script: string) {
  const values: Record<string, string> = {
    title: 'Preview',
    workoutCount: '5',
    totalCalories: '2000',
    weight: '70.5',
  }

  const simpleFields = script.matchAll(/(\w+)\s*:\s*(['"])(.*?)\2/g)
  for (const match of simpleFields) {
    values[match[1]] = match[3]
  }

  const numberFields = script.matchAll(/(\w+)\s*:\s*(-?\d+(?:\.\d+)?)/g)
  for (const match of numberFields) {
    values[match[1]] = match[2]
  }

  return values
}

function titleFromComponentName(name: string) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/[-_]/g, ' ')
}

function attrsToText(attrs: string, values: Record<string, string>) {
  const parts: string[] = []
  const attrMatches = attrs.matchAll(/(?::)?([\w-]+)=["']([^"']+)["']/g)

  for (const match of attrMatches) {
    const key = match[1]
    const rawValue = match[2]
    const value = values[rawValue] || rawValue

    if (key === 'class' || key === 'style') continue
    parts.push(`${key}: ${value}`)
  }

  return parts.join(' · ')
}

function componentPlaceholder(name: string, attrs: string, values: Record<string, string>) {
  const detail = attrsToText(attrs, values)
  return `<section class="component-card">
    <div class="component-title">${escapeHtml(titleFromComponentName(name))}</div>
    ${detail ? `<div class="component-detail">${escapeHtml(detail)}</div>` : ''}
  </section>`
}

function escapeHtml(value: string) {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
}

function stripVueDirectives(template: string) {
  return template
    .replace(/\s+v-\w+(?::[\w-]+)?=(["']).*?\1/g, '')
    .replace(/\s+@\w+(?::[\w-]+)?=(["']).*?\1/g, '')
    .replace(/\s+:\w+=(["']).*?\1/g, '')
}

function replaceRouterLinks(template: string) {
  return template
    .replace(
      /<router-link([^>]*)to=["']([^"']+)["']([^>]*)>([\s\S]*?)<\/router-link>/gi,
      (_, before: string, to: string, after: string, label: string) => {
        return `<a${before}${after} href="${escapeHtml(to)}" data-route="${escapeHtml(to)}">${label}</a>`
      },
    )
    .replace(
      /<RouterLink([^>]*)to=["']([^"']+)["']([^>]*)>([\s\S]*?)<\/RouterLink>/g,
      (_, before: string, to: string, after: string, label: string) => {
        return `<a${before}${after} href="${escapeHtml(to)}" data-route="${escapeHtml(to)}">${label}</a>`
      },
    )
}

function renderMustaches(template: string, values: Record<string, string>) {
  return template.replace(/\{\{\s*([\w.]+)\s*\}\}/g, (_, key: string) => {
    const shortKey = key.split('.').pop() || key
    return escapeHtml(values[key] || values[shortKey] || shortKey)
  })
}

function replaceSelfClosingComponents(template: string, values: Record<string, string>) {
  return template.replace(/<([A-Z][\w.-]*)([^>]*)\/>/g, (_, name: string, attrs: string) => {
    return componentPlaceholder(name, attrs, values)
  })
}

function replacePairedEmptyComponents(template: string, values: Record<string, string>) {
  return template.replace(/<([A-Z][\w.-]*)([^>]*)>\s*<\/\1>/g, (_, name: string, attrs: string) => {
    return componentPlaceholder(name, attrs, values)
  })
}

function buildFallbackHtml(values: Record<string, string>) {
  return `<main class="generated-app">
    <section class="hero">
      <p class="eyebrow">Generated Preview</p>
      <h1>${escapeHtml(values.title || 'Preview')}</h1>
      <p>Live preview generated from the selected Vue file.</p>
    </section>
    <section class="grid">
      <article class="metric"><span>Workouts</span><strong>${escapeHtml(values.workoutCount || '5')}</strong></article>
      <article class="metric"><span>Calories</span><strong>${escapeHtml(values.totalCalories || '2000')}</strong></article>
      <article class="metric"><span>Weight</span><strong>${escapeHtml(values.weight || '70.5')}</strong></article>
    </section>
  </main>`
}

function buildSrcdoc(code: string) {
  error.value = null

  const template = extractBlock(code, 'template')
  const script = extractBlock(code, 'script')
  const style = extractBlock(code, 'style')
  const values = extractDataValues(script)

  if (!template) {
    error.value = 'No <template> block found in selected Vue file.'
    return ''
  }

  let html = template
  html = replaceRouterLinks(html)
  html = renderMustaches(html, values)
  html = replaceSelfClosingComponents(html, values)
  html = replacePairedEmptyComponents(html, values)
  html = stripVueDirectives(html)

  if (!html.trim()) {
    html = buildFallbackHtml(values)
  }

  const bridgeScript = `<scr` + `ipt>
      window.__ROUTES__ = ${JSON.stringify(props.routes || [])};
      document.addEventListener('click', function(event) {
        var target = event.target && event.target.closest ? event.target.closest('a[href], [data-route]') : null;
        if (!target) return;
        var route = target.getAttribute('data-route') || target.getAttribute('href');
        if (!route || route.indexOf('http') === 0 || route.indexOf('#') === 0) return;
        event.preventDefault();
        window.parent.postMessage({ type: 'atoms-preview-route', route: route }, '*');
      });
    </scr` + `ipt>`

  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <style>
      * { box-sizing: border-box; }
      body {
        margin: 0;
        min-height: 100vh;
        background: #09090b;
        color: #f4f4f5;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      #app { min-height: 100vh; padding: 24px; }
      h1, h2, h3, p { margin-top: 0; }
      button, input, select {
        border-radius: 6px;
        border: 1px solid #3f3f46;
        background: #18181b;
        color: #f4f4f5;
        padding: 8px 10px;
      }
      button {
        background: #8b5cf6;
        border-color: #8b5cf6;
        cursor: pointer;
      }
      form, .dashboard, .workout-log, .diet-tracker, .weight-tracker, #app > div {
        display: grid;
        gap: 16px;
      }
      nav {
        display: flex;
        gap: 10px;
        padding: 10px;
        margin-bottom: 18px;
        border: 1px solid #27272a;
        border-radius: 8px;
        background: #18181b;
      }
      a { color: #c4b5fd; text-decoration: none; }
      a:hover { color: #ddd6fe; text-decoration: underline; }
      .component-card, .metric-card, .recent-logs, .weight-chart {
        border: 1px solid #27272a;
        border-radius: 8px;
        background: #18181b;
        padding: 16px;
      }
      .component-title {
        font-size: 13px;
        font-weight: 700;
        color: #ddd6fe;
        text-transform: uppercase;
        letter-spacing: .04em;
      }
      .component-detail {
        margin-top: 8px;
        color: #a1a1aa;
        font-size: 13px;
      }
      .hero {
        border: 1px solid #27272a;
        border-radius: 10px;
        padding: 22px;
        background: #18181b;
      }
      .eyebrow { color: #a78bfa; font-size: 12px; text-transform: uppercase; letter-spacing: .08em; }
      .grid {
        display: grid;
        grid-template-columns: repeat(3, minmax(0, 1fr));
        gap: 12px;
      }
      .metric {
        border: 1px solid #27272a;
        border-radius: 8px;
        padding: 16px;
        background: #18181b;
      }
      .metric span { display: block; color: #a1a1aa; font-size: 12px; }
      .metric strong { display: block; margin-top: 6px; font-size: 24px; }
      ${style}
    </style>
  </head>
  <body>
    ${bridgeScript}
    <div id="app">${html}</div>
  </body>
</html>`
}

watch(
  () => props.code,
  (code) => {
    srcdoc.value = code ? buildSrcdoc(code) : ''
  },
  { immediate: true },
)
</script>

<template>
  <div class="h-full">
    <div v-if="error" class="p-4 text-red-400 text-sm">
      <AlertCircle class="w-4 h-4 inline mr-2" />
      {{ error }}
    </div>
    <iframe
      v-else-if="srcdoc"
      :srcdoc="srcdoc"
      sandbox=""
      class="h-full w-full border-0 bg-zinc-950"
      title="Generated app preview"
    />
    <div v-else class="flex items-center justify-center h-full text-zinc-500">
      <Loader2 class="w-6 h-6 animate-spin" />
    </div>
  </div>
</template>
