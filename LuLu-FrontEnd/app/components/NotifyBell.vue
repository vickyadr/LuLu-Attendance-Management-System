<script setup>
import { useNotifyStore } from '~/store/notify'
import {
    faBell, faCircleCheck, faCircleExclamation, faCircleInfo,
    faTriangleExclamation, faTrashCan, faCheckDouble,
} from '@fortawesome/free-solid-svg-icons'

// The notification history next to the signed-in user. Reads the same store the
// toasts push into, so the bell is a record of what actually happened rather
// than a second, separately maintained feed.
const notify = useNotifyStore()

const open = ref(false)
const rootRef = ref(null)

const ICONS = {
    success: faCircleCheck,
    error: faCircleExclamation,
    info: faCircleInfo,
    warning: faTriangleExclamation,
}
const ICON_TONE = {
    success: 'text-emerald-600',
    error: 'text-red-600',
    info: 'text-emerald-600',
    warning: 'text-amber-600',
}

// Same relative time on both sides of the app; day boundaries matter because the
// history survives across days in localStorage.
function relTime(at) {
    const s = Math.floor((Date.now() - at) / 1000)
    if (s < 60) return 'just now'
    const m = Math.floor(s / 60)
    if (m < 60) return `${m}m ago`
    const h = Math.floor(m / 60)
    if (h < 24) return `${h}h ago`
    const d = Math.floor(h / 24)
    if (d < 7) return `${d}d ago`
    return new Date(at).toLocaleDateString()
}

function toggle() {
    open.value = !open.value
    // Opening is the read receipt: the badge is about "unseen since last look".
    if (open.value) notify.markAllRead()
}

function onDocClick(e) {
    if (!open.value) return
    const el = rootRef.value
    if (el && !el.contains(e.target)) open.value = false
}
function onKey(e) {
    if (e.key === 'Escape') open.value = false
}

onMounted(() => {
    notify.hydrate()
    document.addEventListener('click', onDocClick)
    window.addEventListener('keydown', onKey)
})
onBeforeUnmount(() => {
    document.removeEventListener('click', onDocClick)
    window.removeEventListener('keydown', onKey)
})
</script>

<template>
    <div ref="rootRef" class="relative shrink-0">
        <button type="button" @click.stop="toggle" :aria-expanded="open ? 'true' : 'false'" aria-haspopup="true"
            :aria-label="$t('notify.title')"
            class="relative w-9 h-9 grid place-content-center rounded-xl text-emerald-700 hover:bg-emerald-50 active:bg-emerald-100 transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-emerald-500">
            <font-awesome :icon="faBell" class="w-4 h-4" />
            <!-- Badge is aria-hidden; the button's label carries the count for AT. -->
            <span v-if="notify.unreadCount" aria-hidden="true"
                class="absolute -top-0.5 -end-0.5 min-w-[1.05rem] h-[1.05rem] px-1 rounded-full bg-emerald-600 text-white text-[9px] font-bold grid place-content-center ring-2 ring-white">
                {{ notify.unreadCount > 9 ? '9+' : notify.unreadCount }}
            </span>
        </button>

        <!-- Fixed to the viewport on mobile: the bell is not the last item in the
             header row, so bell-anchored `right-0` pushed the panel past the left
             edge. Anchored to the bell again from `sm` up. -->
        <div v-if="open" class="fixed right-3 top-[4.5rem] z-50 w-[min(22rem,calc(100vw-1.5rem))]
                    rounded-2xl bg-white border border-emerald-100 shadow-[0_12px_32px_-12px_rgba(2,44,34,.28)] overflow-hidden
                    sm:absolute sm:right-0 sm:top-full sm:mt-2 sm:w-[min(22rem,calc(100vw-2rem))]">
            <div class="flex items-center gap-2 px-4 py-3 bg-emerald-50/60 border-b border-emerald-100">
                <font-awesome :icon="faBell" class="w-3.5 h-3.5 text-emerald-600 shrink-0" />
                <p class="text-xs font-bold text-green-900">{{ $t('notify.title') }}</p>
                <span class="ml-auto mono text-[10px] text-emerald-700/60">{{ notify.history.length }}</span>
            </div>

            <div class="max-h-[60vh] overflow-y-auto">
                <p v-if="!notify.orderedHistory.length" class="px-4 py-8 text-center text-xs text-emerald-700/60">
                    {{ $t('notify.empty') }}
                </p>
                <div v-for="n in notify.orderedHistory" :key="n.id"
                    class="flex items-start gap-2.5 px-4 py-3 border-b border-emerald-50 last:border-b-0"
                    :class="n.read ? 'bg-white' : 'bg-emerald-50/40'">
                    <font-awesome :icon="ICONS[n.kind] || ICONS.info" class="w-3.5 h-3.5 mt-0.5 shrink-0"
                        :class="ICON_TONE[n.kind] || ICON_TONE.info" />
                    <div class="min-w-0 flex-1">
                        <p class="text-xs font-semibold text-green-900 leading-snug break-words">{{ n.title }}</p>
                        <p v-if="n.message" class="text-[11px] text-emerald-700/70 leading-snug mt-0.5 break-words">{{
                            n.message }}</p>
                        <p class="mono text-[9px] tracking-wider text-emerald-700/40 mt-1">{{ relTime(n.at) }}</p>
                    </div>
                    <span v-if="!n.read" aria-hidden="true"
                        class="w-1.5 h-1.5 rounded-full bg-emerald-500 mt-1.5 shrink-0"></span>
                </div>
            </div>

            <div class="flex items-center gap-2 px-3 py-2 border-t border-emerald-100 bg-emerald-50/40">
                <button type="button" @click="notify.markAllRead"
                    class="flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold text-emerald-700 hover:bg-emerald-100 transition">
                    <font-awesome :icon="faCheckDouble" class="w-3 h-3" />
                    <span>{{ $t('notify.markAllRead') }}</span>
                </button>
                <button type="button" @click="notify.clearHistory"
                    class="ms-auto flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold text-red-600 hover:bg-red-50 transition">
                    <font-awesome :icon="faTrashCan" class="w-3 h-3" />
                    <span>{{ $t('common.clear') }}</span>
                </button>
            </div>
        </div>
    </div>
</template>
