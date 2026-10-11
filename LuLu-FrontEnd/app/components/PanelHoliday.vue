<script setup>
import { useHolidayStore } from '~/store/holiday'
import { useNotifyStore } from '~/store/notify'
import { faCalendarPlus, faCloudArrowDown, faPen, faTrash } from '@fortawesome/free-solid-svg-icons'
import { fetchHolidayCalendar } from '~/utils/holidayIcs'

const holiday = useHolidayStore()
const notify = useNotifyStore()
const { t } = useI18n()

const KINDS = [
    { value: 'national', label: 'holiday.kindNational' },
    { value: 'collective', label: 'holiday.kindCollective' },
    { value: 'office', label: 'holiday.kindOffice' },
]

// Year filter. The calendar is empty on first run, so the list must default to
// a year the operator is likely to care about rather than an empty string.
const year = ref(String(new Date().getFullYear()))
const editDate = ref('')
const editName = ref('')
const editKind = ref('national')
const editIncluded = ref(false)
const editing = ref(false)
// The form lives in a dialog: inline below the table it sat off-screen on a
// phone with nothing to indicate it existed.
const formOpen = ref(false)

const importing = ref(false)
const importRows = ref([])
const importNote = ref('')
const importError = ref('')

const canEdit = computed(() => /^(\d{4})-(\d{2})-(\d{2})$/.test(editDate.value) && editName.value.trim().length > 0)

const years = computed(() => {
    const set = new Set(holiday.items.map((h) => h.holiday_date.slice(0, 4)))
    const now = String(new Date().getFullYear())
    set.add(now)
    const next = String(Number(now) + 1)
    set.add(next)
    return [...set].sort()
})

const visible = computed(() => holiday.items.filter((h) => !year.value || h.holiday_date.startsWith(year.value)))

function kindLabel(kind) {
    return KINDS.find((k) => k.value === kind)?.label || 'holiday.kindOffice'
}

function kindChip(kind) {
    if (kind === 'national') return 'chip-emerald'
    if (kind === 'collective') return 'chip-lime'
    return 'chip-dark'
}

function startEdit(row) {
    editing.value = true
    formOpen.value = true
    editDate.value = row.holiday_date
    editName.value = row.holiday_name
    editKind.value = row.holiday_kind
    editIncluded.value = !!row.holiday_included
}

function openAdd() {
    editing.value = false
    resetForm()
    formOpen.value = true
}

function resetForm() {
    editing.value = false
    formOpen.value = false
    editDate.value = ''
    editName.value = ''
    editKind.value = 'national'
    editIncluded.value = false
}

// Flip counting on or off for one date without opening the form. Reuses the same
// upsert the form does, so the two paths cannot drift on what they write.
async function toggleIncluded(row) {
    const next = !row.holiday_included
    const resp = await holiday.saveHoliday({
        holiday_date: row.holiday_date,
        holiday_name: row.holiday_name,
        holiday_kind: row.holiday_kind,
        holiday_included: next,
    })
    if (resp.code === 200) notify.success(next ? t('holiday.includedOn') : t('holiday.includedOff'), row.holiday_name)
    else notify.error(t('holiday.saveFailed'), row.holiday_date)
}

async function load() {
    const items = await holiday.fetchHolidays()
    if (!items.length && holiday.loaded) {
        // Not an error: a fresh install simply has no calendar yet. The empty
        // state explains what to do next.
        return
    }
}

async function save() {
    if (!canEdit.value) return
    const resp = await holiday.saveHoliday({
        holiday_date: editDate.value,
        holiday_name: editName.value.trim(),
        holiday_kind: editKind.value,
        holiday_included: editIncluded.value,
    })
    if (resp.code === 200) {
        notify.success(t('holiday.saved'), editName.value.trim())
        resetForm()
    } else {
        notify.error(t('holiday.saveFailed'), Object.values(resp.data || {}).join(' '))
    }
}

async function remove(row) {
    if (!confirm(t('holiday.confirmDelete', { date: row.holiday_date }))) return
    const resp = await holiday.deleteHoliday(row.holiday_date)
    if (resp.code === 200) notify.success(t('holiday.deleted'), row.holiday_name)
    else notify.error(t('holiday.deleteFailed'), row.holiday_date)
}

// One-click refresh from the public Indonesian calendar. The rows are shown
// first and only written when the operator confirms, because the feed is
// unofficial and may carry a wrong or missing date.
async function runImport() {
    importing.value = true
    importError.value = ''
    importNote.value = ''
    importRows.value = []
    try {
        const { rows, timedObservances } = await fetchHolidayCalendar()
        importRows.value = rows
        if (timedObservances) importNote.value = `timed:${timedObservances}`
    } catch (e) {
        // 'timeout' and 'fetch' are distinct operator-facing problems; 'empty'
        // means the calendar answered but carried nothing usable.
        importError.value = ['timeout', 'fetch', 'empty'].includes(e?.message) ? e.message : 'fetch'
    } finally {
        importing.value = false
    }
}

async function applyImport() {
    const rows = importRows.value
    if (!rows.length) return
    const resp = await holiday.importHolidays(rows)
    if (resp.code === 200) {
        notify.success(t('holiday.imported', { count: rows.length }), '')
        importRows.value = []
        year.value = rows[0].holiday_date.slice(0, 4)
    } else {
        notify.error(t('holiday.saveFailed'), Object.values(resp.data || {}).join(' '))
    }
}

onMounted(load)
</script>

<template>
    <div class="hud-frame glass rounded-2xl border border-emerald-100 p-4 sm:p-5 relative overflow-hidden">
        <span class="hud-corner hud-corner-tl hidden sm:block"></span><span
            class="hud-corner hud-corner-br hidden sm:block"></span>

        <div class="flex flex-col sm:flex-row sm:items-start justify-between gap-3">
            <div>
                <h2 class="display text-base font-extrabold text-green-900 tracking-tight">{{ $t('holiday.title') }}
                </h2>
            </div>
            <div class="flex items-center gap-2">
                <select v-model="year"
                    class="h-9 rounded-lg border-2 border-emerald-100 bg-white px-2 mono text-xs text-green-900 focus:outline-none focus:border-emerald-400"
                    :aria-label="$t('common.filter')">
                    <option value="">{{ $t('common.all') }}</option>
                    <option v-for="y in years" :key="y" :value="y">{{ y }}</option>
                </select>
                <button type="button"
                    class="h-9 px-3 rounded-lg border-2 border-emerald-100 bg-white text-xs font-bold text-green-900 hover:bg-emerald-50 transition inline-flex items-center gap-2"
                    @click="openAdd">
                    <font-awesome :icon="faCalendarPlus" class="w-3.5 h-3.5" />
                    <span>{{ $t('holiday.addBtn') }}</span>
                </button>
                <button type="button"
                    class="btn-emerald h-9 px-3 rounded-lg text-xs font-bold inline-flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed transition"
                    :disabled="importing" @click="runImport">
                    <span v-if="importing"
                        class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                    <font-awesome v-else :icon="faCloudArrowDown" class="w-3.5 h-3.5" />
                    <span>{{ $t('holiday.importBtn') }}</span>
                </button>
            </div>
        </div>

        <!-- Only one line of guidance: a holiday silences everyone that day, which
             is not guessable from the label alone -->
        <p
            class="mt-3 rounded-xl bg-emerald-50/60 border border-emerald-100 px-3 py-2 mono text-[11px] text-emerald-800">
            {{ $t('holiday.howItWorks') }}
        </p>

        <!-- Import failure is always visible, never a silent no-op -->
        <div v-if="importError"
            class="mt-3 rounded-xl bg-amber-50 border border-amber-200 px-3 py-2 mono text-[11px] text-amber-800">
            {{ $t(importError === 'timeout' ? 'holiday.importTimeout' : importError === 'empty' ?
                'holiday.importNothing' : 'holiday.importFailed') }}
        </div>

        <!-- Import preview: nothing is written until confirmed -->
        <div v-if="importRows.length" class="mt-3 rounded-xl border border-emerald-200 bg-white px-3 py-3">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <span class="mono text-[11px] font-bold tracking-widest text-emerald-800">{{ $t('holiday.importPreview')
                    }}</span>
                <div class="flex items-center gap-2">
                    <button type="button"
                        class="h-9 px-3 rounded-lg border-2 border-emerald-100 bg-white text-xs font-bold text-green-900 hover:bg-emerald-50 transition"
                        @click="importRows = []">
                        {{ $t('holiday.cancelBtn') }}
                    </button>
                    <button type="button"
                        class="btn-emerald h-9 px-3 rounded-lg text-xs font-bold disabled:opacity-40 disabled:cursor-not-allowed transition"
                        :disabled="holiday.saving" @click="applyImport">
                        {{ $t('holiday.importApply', { count: importRows.length }) }}
                    </button>
                </div>
            </div>
            <div class="mt-2 max-h-48 overflow-y-auto scrollbar-thin space-y-1">
                <div v-for="row in importRows" :key="row.holiday_date"
                    class="flex items-center gap-2 mono text-[11px] text-green-900">
                    <span class="chip shrink-0" :class="kindChip(row.holiday_kind)">{{ $t(kindLabel(row.holiday_kind))
                        }}</span>
                    <span class="w-24 shrink-0 text-emerald-700/70">{{ row.holiday_date }}</span>
                    <span class="truncate">{{ row.holiday_name }}</span>
                </div>
            </div>
        </div>

        <!-- List: same layout as the shift table (gradient sticky header, hover rows,
             square icon buttons, separate mobile cards) -->
        <div class="hidden sm:block overflow-auto max-h-[360px]">
            <table class="w-full">
                <thead class="sticky top-0 z-10">
                    <tr class="bg-gradient-to-r from-emerald-600 to-green-600 text-white">
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                            {{ $t('holiday.colDate') }}</th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                            {{ $t('holiday.colName') }}</th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                            {{ $t('holiday.colKind') }}</th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 hidden lg:table-cell">
                            {{ $t('holiday.colSource') }}</th>
                        <th
                            class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                            {{ $t('holiday.colCounted') }}</th>
                        <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase">{{
                            $t('common.actions') }}</th>
                    </tr>
                </thead>
                <tbody v-if="visible.length > 0" class="divide-y divide-emerald-50">
                    <tr v-for="row in visible" :key="row.holiday_date"
                        class="group hover:bg-emerald-50/50 transition text-xs">
                        <td class="px-3 py-2.5 mono text-green-900 whitespace-nowrap">{{ row.holiday_date }}</td>
                        <td class="px-3 py-2.5 font-bold text-green-900">{{ row.holiday_name }}</td>
                        <td class="px-3 py-2.5">
                            <span class="chip" :class="kindChip(row.holiday_kind)">{{ $t(kindLabel(row.holiday_kind))
                                }}</span>
                        </td>
                        <td class="px-3 py-2.5 mono text-[11px] text-emerald-700/70 hidden lg:table-cell">
                            {{ row.holiday_source === 'import' ? $t('holiday.srcImport') : $t('holiday.srcManual') }}
                        </td>
                        <td class="px-3 py-2.5 text-center">
                            <button type="button" role="switch" :aria-checked="row.holiday_included ? 'true' : 'false'"
                                :aria-label="$t('holiday.colCounted')" :title="$t('holiday.countedHint')"
                                class="relative w-9 h-5 rounded-full transition disabled:opacity-40"
                                :class="row.holiday_included ? 'bg-emerald-500' : 'bg-emerald-200'"
                                :disabled="holiday.saving" @click="toggleIncluded(row)">
                                <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all"
                                    :class="row.holiday_included ? 'left-4' : 'left-0.5'"></span>
                            </button>
                        </td>
                        <td class="px-3 py-2.5 text-center">
                            <div class="inline-flex items-center gap-1">
                                <button type="button"
                                    class="w-7 h-7 rounded-lg bg-white border border-emerald-100 text-emerald-600 hover:bg-emerald-600 hover:text-white hover:border-emerald-600 flex items-center justify-center transition"
                                    :title="$t('common.edit')" @click="startEdit(row)">
                                    <font-awesome :icon="faPen" class="w-3 h-3" />
                                </button>
                                <button type="button"
                                    class="w-7 h-7 rounded-lg bg-white border border-red-100 text-red-500 hover:bg-red-500 hover:text-white hover:border-red-500 flex items-center justify-center transition"
                                    :title="$t('common.delete')" @click="remove(row)">
                                    <font-awesome :icon="faTrash" class="w-3 h-3" />
                                </button>
                            </div>
                        </td>
                    </tr>
                </tbody>
                <tbody v-else>
                    <tr>
                        <td colspan="6" class="py-8 text-center">
                            <p class="mono text-xs text-emerald-700/50">{{ $t('holiday.empty') }}</p>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Mobile cards -->
        <div class="sm:hidden p-2 space-y-2 max-h-[360px] overflow-auto">
            <div v-if="visible.length === 0"
                class="py-8 text-center rounded-xl bg-emerald-50/40 border border-emerald-100 mono text-xs text-emerald-700/60">
                {{ $t('holiday.empty') }}</div>
            <div v-for="row in visible" :key="row.holiday_date"
                class="rounded-2xl border-2 border-emerald-100 bg-white p-3 flex items-center gap-3 active:border-emerald-300 active:bg-emerald-50/40">
                <span class="w-2 h-10 rounded-full shrink-0"
                    :class="row.holiday_included ? 'bg-emerald-500' : 'bg-emerald-200'"></span>
                <div class="flex-1 min-w-0">
                    <p class="font-bold text-sm text-green-900 truncate">{{ row.holiday_name }}</p>
                    <p class="mono text-xs text-emerald-700">{{ row.holiday_date }} <span
                            class="text-emerald-700/40">•</span> {{ $t(kindLabel(row.holiday_kind)) }}</p>
                </div>
                <button type="button" role="switch" :aria-checked="row.holiday_included ? 'true' : 'false'"
                    :aria-label="$t('holiday.colCounted')"
                    class="relative w-9 h-5 rounded-full transition shrink-0 disabled:opacity-40"
                    :class="row.holiday_included ? 'bg-emerald-500' : 'bg-emerald-200'" :disabled="holiday.saving"
                    @click="toggleIncluded(row)">
                    <span class="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all"
                        :class="row.holiday_included ? 'left-4' : 'left-0.5'"></span>
                </button>
                <div class="flex flex-col gap-1 shrink-0">
                    <button type="button" @click="startEdit(row)"
                        class="px-3 py-1.5 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold">{{
                        $t('common.edit') }}</button>
                    <button type="button" @click="remove(row)"
                        class="px-3 py-1.5 rounded-full bg-white border border-red-200 text-red-600 mono text-[10px]">{{
                            $t('common.delete') }}</button>
                </div>
            </div>
        </div>
    </div>

    <ModalHolidayForm v-model:date="editDate" v-model:name="editName" v-model:kind="editKind"
        v-model:included="editIncluded" :show="formOpen" :editing="editing" :saving="holiday.saving" :can-save="canEdit"
        :kinds="KINDS" @close="resetForm" @save="save" />
</template>
