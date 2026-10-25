<script setup>
import { useNotifyStore } from '~/store/notify'
import {
    faCircleCheck, faCircleExclamation, faCircleInfo, faTriangleExclamation,
    faXmark,
} from '@fortawesome/free-solid-svg-icons'

// Renders the live toasts. Mounted once in the layout, so any page can push to
// it through the store without owning markup.
const notify = useNotifyStore()

const ICONS = {
    success: faCircleCheck,
    error: faCircleExclamation,
    info: faCircleInfo,
    warning: faTriangleExclamation,
}

// Emerald for success reads as the app's own colour; the rest stay readable
// rather than introducing a second brand palette.
const TONE = {
    success: 'border-border-strong bg-surface text-text',
    error: 'border-red-200 bg-surface text-danger',
    info: 'border-border bg-surface text-text',
    warning: 'border-amber-200 bg-surface text-warn',
}
const ICON_TONE = {
    success: 'text-primary-deep',
    error: 'text-danger',
    info: 'text-primary-deep',
    warning: 'text-warn',
}
</script>

<template>
    <!-- aria-live so a screen reader announces the toast without stealing focus -->
    <div class="fixed bottom-4 right-4 z-[120] flex flex-col gap-2 w-[min(22rem,calc(100vw-2rem))] pointer-events-none"
        role="status" aria-live="polite">
        <TransitionGroup name="toast">
            <div v-for="t in notify.toasts" :key="t.id"
                class="pointer-events-auto flex items-start gap-3 rounded-xl border px-4 py-3 shadow-tint-lg"
                :class="TONE[t.kind] || TONE.info">
                <font-awesome :icon="ICONS[t.kind] || ICONS.info" class="w-4 h-4 mt-1 shrink-0"
                    :class="ICON_TONE[t.kind] || ICON_TONE.info" />
                <div class="min-w-0 flex-1">
                    <p class="text-[13px] font-bold leading-snug">{{ t.title }}</p>
                    <p v-if="t.message" class="text-xs leading-snug text-text-muted mt-0.5 break-words">{{ t.message }}
                    </p>
                </div>
                <button type="button" @click="notify.dismiss(t.id)" :aria-label="$t('common.dismiss')"
                    class="shrink-0 w-8 h-8 -mr-1.5 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised hover:text-text transition">
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
    transform: translateX(12px) translateY(4px);
}

.toast-leave-to {
    opacity: 0;
    transform: translateX(12px) translateY(4px);
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
