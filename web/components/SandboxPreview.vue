<script setup lang="ts">
import type { FileNode } from '~/types/apps'

const props = defineProps<{
  // Whole generated project. Only `type: 'file'` nodes with content are run.
  files: FileNode[]
  // Optional explicit entry path; otherwise auto-detected.
  entry?: string
}>()

const srcdoc = ref('')
const empty = ref(false)
const iframeReady = ref(false)

// Pick the module that should bootstrap the app.
// Prefer a self-mounting entry (main.*) so router/plugin wiring is preserved,
// then fall back to a root component.
function detectEntry(fileMap: Record<string, string>): string | null {
  const keys = Object.keys(fileMap)
  const prefer = [
    'src/main.ts', 'src/main.js', 'src/main.mjs',
    'main.ts', 'main.js', 'main.mjs',
    'src/App.vue', 'App.vue',
    'src/pages/index.vue', 'pages/index.vue',
  ]
  for (const p of prefer) {
    if (fileMap[p] != null) return p
  }
  return keys.find(k => k.endsWith('.vue')) || keys.find(k => /\.(ts|js|mjs)$/.test(k)) || null
}

function buildSrcdoc() {
  empty.value = false

  const fileMap: Record<string, string> = {}
  for (const node of props.files || []) {
    if (node.type === 'file' && typeof node.content === 'string') {
      fileMap[node.path.replace(/^\.?\//, '')] = node.content
    }
  }

  const entry = (props.entry && fileMap[props.entry.replace(/^\.?\//, '')] != null)
    ? props.entry.replace(/^\.?\//, '')
    : detectEntry(fileMap)

  if (!entry) {
    empty.value = true
    srcdoc.value = ''
    return
  }

  // Embed payload as escaped JSON: generated SFC content contains literal
  // script-closing tags, so escape every "<" to stop the host script tag
  // from closing early.
  const payload = JSON.stringify({ files: fileMap, entry }).replace(/</g, '\\u003c')
  const origin = window.location.origin

  srcdoc.value = `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width,initial-scale=1" />
    <style>
      * { box-sizing: border-box; }
      html, body, #app { height: 100%; margin: 0; }
      body {
        background: #09090b;
        color: #f4f4f5;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      #__preview_error__ {
        display: none;
        padding: 16px;
        margin: 16px;
        border: 1px solid #7f1d1d;
        border-radius: 8px;
        background: #1f0f12;
        color: #fca5a5;
        font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
        font-size: 12px;
        white-space: pre-wrap;
        word-break: break-word;
      }
    </style>
    <script src="${origin}/preview-runtime/vue.global.prod.js"><\/script>
    <script src="${origin}/preview-runtime/vue-router.global.prod.js"><\/script>
    <script src="${origin}/preview-runtime/vue3-sfc-loader.js"><\/script>
  </head>
  <body>
    <div id="app"></div>
    <pre id="__preview_error__"></pre>
    <script>
      // The sandbox withholds 'allow-same-origin' for isolation, which makes the
      // real localStorage/sessionStorage throw. Provide in-memory shims so generated
      // apps that persist state (e.g. a game high score) run instead of crashing.
      (function () {
        function memStorage() {
          var map = {};
          return {
            getItem: function (k) { return Object.prototype.hasOwnProperty.call(map, k) ? map[k] : null; },
            setItem: function (k, v) { map[k] = String(v); },
            removeItem: function (k) { delete map[k]; },
            clear: function () { map = {}; },
            key: function (i) { return Object.keys(map)[i] || null; },
            get length() { return Object.keys(map).length; },
          };
        }
        try {
          Object.defineProperty(window, 'localStorage', { value: memStorage(), configurable: true });
          Object.defineProperty(window, 'sessionStorage', { value: memStorage(), configurable: true });
        } catch (e) { /* ignore */ }
      })();
    <\/script>
    <script type="application/json" id="__vfs__">${payload}<\/script>
    <script>
      (function () {
        var box = document.getElementById('__preview_error__');
        function showError(err) {
          var msg = (err && err.stack) ? err.stack : (err && err.message) ? err.message : String(err);
          box.style.display = 'block';
          box.textContent = 'Preview failed to run:\\n\\n' + msg;
        }
        window.addEventListener('error', function (e) { showError(e.error || e.message); });
        window.addEventListener('unhandledrejection', function (e) { showError(e.reason); });

        try {
          var payload = JSON.parse(document.getElementById('__vfs__').textContent);
          var files = payload.files;
          var entry = payload.entry;

          var Vue = window.Vue;
          var VueRouter = window.VueRouter;
          var sfc = window['vue3-sfc-loader'];
          if (!Vue || !sfc) { throw new Error('Preview runtime failed to load. Check /preview-runtime assets.'); }

          // Keep generated routing fully in-memory: internal navigation works,
          // but nothing touches the URL or escapes the iframe.
          var routerModule = VueRouter ? Object.assign({}, VueRouter, {
            createWebHistory: VueRouter.createMemoryHistory,
            createWebHashHistory: VueRouter.createMemoryHistory,
          }) : undefined;

          var exts = ['', '.vue', '.ts', '.js', '.mjs', '.json', '.css'];
          function resolveFile(raw) {
            var p = String(raw).replace(/^\\.?\\//, '').replace(/^\\/+/, '');
            var candidates = [];
            for (var i = 0; i < exts.length; i++) candidates.push(p + exts[i]);
            candidates.push(p.replace(/\\/?$/, '/index.vue'));
            candidates.push(p.replace(/\\/?$/, '/index.ts'));
            candidates.push(p.replace(/\\/?$/, '/index.js'));
            for (var j = 0; j < candidates.length; j++) {
              if (files[candidates[j]] != null) return candidates[j];
            }
            // Last resort: match by file name regardless of directory.
            var base = p.split('/').pop();
            var keys = Object.keys(files);
            for (var k = 0; k < keys.length; k++) {
              var kb = keys[k].split('/').pop();
              if (kb === base) return keys[k];
              for (var e = 1; e < exts.length; e++) { if (kb === base + exts[e]) return keys[k]; }
            }
            return null;
          }

          // The loader parses '.js' as a SCRIPT (no import/export). Treat JS files
          // as ES modules so generated import/export works; the '.ts' parser is a
          // superset that handles plain JS too.
          function moduleType(ext) {
            return (ext === '.js' || ext === '.jsx' || ext === '.cjs' || ext === '.mjs') ? '.ts' : ext;
          }

          var options = {
            moduleCache: Object.assign({ vue: Vue }, routerModule ? { 'vue-router': routerModule } : {}),
            getFile: function (url) {
              var key = resolveFile(url);
              if (key == null) {
                // Tolerate a hallucinated import (a file the generator referenced
                // but never produced): stub it so one bad import doesn't kill the
                // whole preview.
                var base = String(url).split('/').pop() || '';
                var dotI = base.lastIndexOf('.');
                var ext = dotI > 0 ? base.slice(dotI).toLowerCase() : (/^[A-Z]/.test(base) ? '.vue' : '.js');
                var stub;
                if (ext === '.vue') {
                  stub = '<template><div style="padding:8px;border:1px dashed #52525b;border-radius:6px;color:#a1a1aa;font-size:12px;font-family:monospace">[missing: ' + url + ']</div></template>';
                } else if (ext === '.json') {
                  stub = '{}';
                } else if (ext === '.css') {
                  stub = '';
                } else {
                  stub = 'export default {};';
                }
                console.warn('[preview] missing module stubbed:', url);
                return {
                  getContentData: function (asBinary) {
                    return asBinary ? new TextEncoder().encode(stub).buffer : stub;
                  },
                  type: moduleType(ext),
                };
              }
              var content = files[key];
              var dot = key.lastIndexOf('.');
              var type = dot >= 0 ? key.slice(dot) : '.js';
              return {
                getContentData: function (asBinary) {
                  return asBinary ? new TextEncoder().encode(content).buffer : content;
                },
                type: moduleType(type),
              };
            },
            addStyle: function (textContent) {
              var style = document.createElement('style');
              style.textContent = textContent;
              document.head.appendChild(style);
            },
            handleModule: function (type, getContentData, path) {
              if (type === '.json') return JSON.parse(getContentData(false));
              if (type === '.css') {
                var style = document.createElement('style');
                style.textContent = getContentData(false);
                document.head.appendChild(style);
                return {};
              }
              return undefined;
            },
            log: function (type) {
              if (type === 'error') {
                var rest = Array.prototype.slice.call(arguments, 1).join(' ');
                showError(rest);
              }
            },
          };

          sfc.loadModule(entry, options).then(function (mod) {
            var comp = mod && (mod.default || mod);
            var isComponent = comp && (typeof comp === 'object' || typeof comp === 'function') &&
              (comp.render || comp.setup || comp.template || comp.__file || comp.components || comp.data || comp.props);
            // A self-mounting entry (main.*) mounts itself and exports nothing useful.
            if (isComponent) {
              var app = Vue.createApp(comp);
              app.config.errorHandler = showError;
              app.mount('#app');
            }
            window.parent.postMessage({ type: '__preview_ready__' }, '*');
          }).catch(function (err) {
            showError(err);
            window.parent.postMessage({ type: '__preview_ready__' }, '*');
          });
        } catch (err) {
          showError(err);
          window.parent.postMessage({ type: '__preview_ready__' }, '*');
        }
      })();
    <\/script>
  </body>
</html>`
}

watch(
  () => [props.files, props.entry],
  () => {
    iframeReady.value = false
    buildSrcdoc()
  },
  { immediate: true, deep: true },
)

function onPreviewMessage(event: MessageEvent) {
  if (event.data?.type === '__preview_ready__') {
    iframeReady.value = true
  }
}

onMounted(() => window.addEventListener('message', onPreviewMessage))
onBeforeUnmount(() => window.removeEventListener('message', onPreviewMessage))
</script>

<template>
  <div class="h-full relative">
    <div v-if="empty" class="flex items-center justify-center h-full text-zinc-500 text-sm">
      <AlertCircle class="w-4 h-4 inline mr-2" />
      No runnable Vue entry found in generated files.
    </div>
    <template v-else-if="srcdoc">
      <iframe
        :srcdoc="srcdoc"
        sandbox="allow-scripts allow-forms allow-modals allow-popups"
        class="h-full w-full border-0 bg-zinc-950"
        title="Generated app preview"
      />
      <Transition name="fade">
        <div
          v-if="!iframeReady"
          class="absolute inset-0 flex flex-col items-center justify-center bg-zinc-950/90 z-10"
        >
          <Loader2 class="w-6 h-6 text-violet-400 animate-spin mb-3" />
          <p class="text-xs text-zinc-500">Compiling preview…</p>
        </div>
      </Transition>
    </template>
    <div v-else class="flex items-center justify-center h-full text-zinc-500">
      <Loader2 class="w-6 h-6 animate-spin" />
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
    </div>
  </div>
</template>
