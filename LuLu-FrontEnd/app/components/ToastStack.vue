<script setup>
import { useNotifyStore } from '~/store/notify'
import {
    faCircleCheck, faCircleExclamation, faCircleInfo, faTriangleExclamation,
    faXmark,
} from '@fortawesome/free-solid-svg-icons'

const notify = useNotifyStore()

const ICONS = {
    success: faCircleCheck,
    error: faCircleExclamation,
    info: faCircleInfo,
    warning: faTriangleExclamation,
}

const TONE = {
    success: 'border-emerald-200 bg-white text-emerald-800',
    error: 'border-red-200 bg-white text-red-800',
    info: 'border-emerald-200 bg-white text-green-900',
    warning: 'border-amber-200 bg-white text-amber-800',
}
const ICON_TONE = {
    success: 'text-emerald-600',
    error: 'text-red-600',
    info: 'text-emerald-600',
    warning: 'text-amber-600',
}
</script>

<template>
    <!-- aria-live so a screen reader announces the toast without stealing focus -->
    <div class="fixed top-20 right-4 z-[120] flex flex-col gap-2 w-[min(22rem,calc(100vw-2rem))] pointer-events-none"
        role="status" aria-live="polite">
        <TransitionGroup name="toast">
            <div v-for="t in notify.toasts" :key="t.id"
                class="pointer-events-auto flex items-start gap-3 rounded-xl border px-4 py-3 shadow-[0_12px_32px_-12px_rgba(2,44,34,.32)]"
                :class="TONE[t.kind] || TONE.info">
                <font-awesome :icon="ICONS[t.kind] || ICONS.info" class="w-4 h-4 mt-0.5 shrink-0"
                    :class="ICON_TONE[t.kind] || ICON_TONE.info" />
                <div class="min-w-0 flex-1">
                    <p class="text-sm font-semibold leading-snug">{{ t.title }}</p>
                    <p v-if="t.message" class="text-xs leading-snug opacity-80 mt-0.5 break-words">{{ t.message }}</p>
                </div>
                <button type="button" @click="notify.dismiss(t.id)" aria-label="Dismiss notification"
                    class="shrink-0 w-6 h-6 grid place-content-center rounded-lg opacity-50 hover:opacity-100 hover:bg-black/5 transition">
                    <font-awesome :icon="faXmark" class="w-3 h-3" />
                </button>
            </div>
        </TransitionGroup>
    </div>
</template>

<style scoped>
/* Enter from the right, leave to the right; the list grows downward. */
.toast-enter-active,
.toast-leave-active {
    transition: opacity .18s ease, transform .18s ease;
}

.toast-enter-from {
    opacity: 0;
    transform: translateX(12px);
}

.toast-leave-to {
    opacity: 0;
    transform: translateX(12px);
}

/* Leave the stack instantly on drop so it does not slide around. */
.toast-leave-active {
    position: absolute;
    right: 0;
    width: 100%;
}

.toast-move {
    transition: transform .18s ease;
}
</style>
