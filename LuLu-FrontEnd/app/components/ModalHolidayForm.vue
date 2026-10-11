<script setup>
/**
 * Holiday add/edit dialog.
 *
 * The form used to sit inline below the table, which hid it: on a phone the
 * panel header alone filled the screen, so reaching the form meant scrolling
 * past 30 rows of holidays, and nothing on screen said a form existed at all.
 * A dialog puts the fields in front of the operator at the moment they asked
 * for them.
 *
 * Every field is bound through `defineModel`, so the parent keeps ownership of
 * the values and this component stays a plain shell with no save logic of its
 * own. The parent decides what closing means.
 */
import { faCalendarPlus, faPen } from '@fortawesome/free-solid-svg-icons'

const props = defineProps({
    show: { type: Boolean, required: true },
    editing: { type: Boolean, default: false },
    saving: { type: Boolean, default: false },
    canSave: { type: Boolean, default: false },
    kinds: { type: Array, required: true },
})

const emit = defineEmits(['close', 'save'])

// Bound through defineModel so the parent keeps ownership of the values: this
// component is a shell with no save logic of its own.
const date = defineModel('date', { type: String, default: '' })
const name = defineModel('name', { type: String, default: '' })
const kind = defineModel('kind', { type: String, default: 'national' })
const included = defineModel('included', { type: Boolean, default: false })

// Close on Escape so the dialog can be dismissed without reaching for the
// button, which matters most on a phone where the X sits far from the thumb.
function onKeydown(e) {
    if (e.key === 'Escape' && props.show && !props.saving) emit('close')
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))

// Move focus into the dialog so the first field is reachable from the keyboard
// and screen readers announce the dialog instead of leaving focus behind it.
const dateInput = ref(null)
watch(
    () => props.show,
    async (open) => {
        if (!open) return
        await nextTick()
        dateInput.value?.focus()
    }
)
</script>

<template>
    <!--
        Unmount with v-if, matching SimpleModal: v-show cannot override Tailwind's
        `block` utility, so a hidden backdrop would keep swallowing clicks on the
        page underneath. `max-h` + overflow keeps the whole dialog reachable on a
        short phone screen instead of running off the bottom; `no-scrollbar` hides
        the bar because a dialog this size almost always fits.
    -->
    <div v-if="show" class="fixed inset-0 z-[100] grid place-items-center bg-green-950/40 backdrop-blur-sm p-4"
        @click.self="!saving && emit('close')">
        <div class="hud-frame w-full max-w-md max-h-[90vh] overflow-y-auto no-scrollbar rounded-[1.25rem] bg-white p-5 shadow-[0_20px_60px_rgba(5,46,22,.25)] border border-emerald-100"
            role="dialog" aria-modal="true" :aria-labelledby="editing ? 'holidayEditTitle' : 'holidayAddTitle'">
            <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>

            <div class="flex items-start justify-between gap-3">
                <h2 :id="editing ? 'holidayEditTitle' : 'holidayAddTitle'"
                    class="display text-lg font-extrabold text-green-900">
                    {{ editing ? $t('holiday.editTitle') : $t('holiday.addTitle') }}
                </h2>
                <button type="button"
                    class="w-8 h-8 shrink-0 rounded-lg border-2 border-emerald-100 bg-white text-emerald-700 hover:bg-emerald-50 transition"
                    :aria-label="$t('common.cancel')" :disabled="saving" @click="emit('close')">
                    <span aria-hidden="true">&times;</span>
                </button>
            </div>

            <div class="mt-4 space-y-3">
                <div>
                    <label for="holiday-date" class="mono text-[10px] tracking-widest text-emerald-700/70">{{
                        $t('holiday.date') }}</label>
                    <input id="holiday-date" ref="dateInput" v-model="date" type="date"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-3 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                </div>

                <div>
                    <label for="holiday-name" class="mono text-[10px] tracking-widest text-emerald-700/70">{{
                        $t('holiday.name') }}</label>
                    <input id="holiday-name" v-model="name" type="text" :placeholder="$t('holiday.namePh')"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-3 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                </div>

                <div>
                    <label for="holiday-kind" class="mono text-[10px] tracking-widest text-emerald-700/70">{{
                        $t('holiday.kind') }}</label>
                    <select id="holiday-kind" v-model="kind"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-2 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                        <option v-for="k in kinds" :key="k.value" :value="k.value">{{ $t(k.label) }}</option>
                    </select>
                </div>

                <label class="flex items-center gap-2 cursor-pointer select-none">
                    <input v-model="included" type="checkbox" class="w-4 h-4 accent-emerald-600">
                    <span class="mono text-[11px] text-emerald-800">{{ $t('holiday.countedLabel') }}</span>
                </label>
            </div>

            <footer class="mt-5 flex justify-end gap-2">
                <button type="button"
                    class="h-10 px-4 rounded-lg border-2 border-emerald-100 bg-white text-sm font-bold text-emerald-700 hover:bg-emerald-50 transition disabled:opacity-40"
                    :disabled="saving" @click="emit('close')">
                    {{ $t('common.cancel') }}
                </button>
                <button type="button"
                    class="btn-emerald h-10 px-4 rounded-lg text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed transition"
                    :disabled="!canSave || saving" @click="emit('save')">
                    <span v-if="saving"
                        class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                    <font-awesome v-else :icon="editing ? faPen : faCalendarPlus" class="w-3.5 h-3.5" />
                    <span>{{ editing ? $t('holiday.saveBtn') : $t('holiday.addBtn') }}</span>
                </button>
            </footer>
        </div>
    </div>
</template>
