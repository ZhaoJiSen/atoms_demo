import { defineNuxtPlugin } from '#app'
import {
  Blocks, Server, Sparkles, ArrowRight, Lightbulb, AlertCircle, Loader2,
  Home, Play, Eye, Plus, MessageSquare, ListChecks, CheckCircle, Circle,
  XCircle, RefreshCw, LayoutGrid, Zap, Database, Folder, File, FileText,
  ChevronRight, ArrowLeft, User, Check, Terminal, StopCircle, Info,
  AlertTriangle, Clock, EyeOff, Wrench,
  Search, ClipboardList, Layout, Code, FileCode,
  LogIn, LogOut, KeyRound, ShieldCheck, Settings, X, Save, Pencil, Trash2
} from '@lucide/vue'

export default defineNuxtPlugin(({ vueApp }) => {
  const components: Record<string, any> = {
    Blocks, Server, Sparkles, ArrowRight, Lightbulb, AlertCircle, Loader2,
    Home, Play, Eye, Plus, MessageSquare, ListChecks, CheckCircle, Circle,
    XCircle, RefreshCw, LayoutGrid, Zap, Database, Folder, File, FileText,
    ChevronRight, ArrowLeft, User, Check, Terminal, StopCircle, Info,
    AlertTriangle, Clock, EyeOff, Wrench,
    Search, ClipboardList, Layout, Code, FileCode,
    LogIn, LogOut, KeyRound, ShieldCheck, Settings, X, Save, Pencil, Trash2
  }

  Object.entries(components).forEach(([name, component]) => {
    vueApp.component(name, component)
  })
})
