<script setup>
import { useHolidayStore } from '~/store/holiday'
import { useNotifyStore } from '~/store/notify'
import { faCalendarPlus, faCloudArrowDown, faInbox, faPen, faTrash } from '@fortawesome/free-solid-svg-icons'
import { fetchHolidayCalendar } from '~/utils/holidayIcs'
import { SMData } from '~/components/SimpleModal.vue'

const holiday = useHolidayStore()
const notify = useNotifyStore()
const { t } = useI18n()
const sm_data = new SMData()

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
    return $t(KINDS.find((k) => k.value === kind)?.label || 'holiday.kindOffice')
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

function remove(row) {
    sm_data.setText(t('holiday.deleteTitle'), t('holiday.confirmDelete', { date: row.holiday_date }))
    sm_data.setHelper('del-holiday', row)
    sm_data.showOKCancel()
}
async function handleDeleteOk() {
    const row = sm_data.getHelper('del-holiday')
    if (row) {
        const resp = await holiday.deleteHoliday(row.holiday_date)
        if (resp.code === 200) notify.success(t('holiday.deleted'), row.holiday_name)
        else notify.error(t('holiday.deleteFailed'), row.holiday_date)
    }
    sm_data.clear()
}
function handleDeleteCancel() { sm_data.clear() }

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
        if (timedObservances) importNote.value = t('holiday.importTimed', { n: timedObservances })
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
    <div class="panel overflow-hidden">

        <div class="panel-head">
            <span class="inline-flex items-center gap-2 min-w-0">
                <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                <span class="truncate">{{ $t('holiday.title') }}</span>
            </span>
            <div class="flex flex-wrap items-center justify-end gap-1.5">
                <select v-model="year" class="field h-9 !w-auto px-2 mono text-xs normal-case"
                    :aria-label="$t('common.filter')">
                    <option value="">{{ $t('common.all') }}</option>
                    <option v-for="y in years" :key="y" :value="y">{{ y }}</option>
                </select>
                <button v-if="year" type="button" @click="year = ''"
                    class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{ $t('common.reset') }}</button>
                <button type="button" class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]" @click="openAdd">
                    <font-awesome :icon="faCalendarPlus" class="w-3.5 h-3.5" />
                    <span>{{ $t('holiday.addBtn') }}</span>
                </button>
                <button type="button" class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                    :disabled="importing" @click="runImport">
                    <span v-if="importing"
                        class="w-3.5 h-3.5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                    <font-awesome v-else :icon="faCloudArrowDown" class="w-3.5 h-3.5" />
                    <span>{{ $t('holiday.importBtn') }}</span>
                </button>
            </div>
        </div>

        <div class="p-3 sm:p-4 space-y-3">

            <!-- Only one line of guidance: a holiday silences everyone that day, which
                 is not guessable from the label alone -->
            <p
                class="rounded-lg bg-surface-raised/60 border border-border px-3 py-2 text-[11.5px] leading-relaxed text-text-muted">
                {{ $t('holiday.howItWorks') }}
            </p>

            <!-- Import failure is always visible, never a silent no-op -->
            <div v-if="importError"
                class="rounded-lg bg-warn-bg border border-amber-200 px-3 py-2 text-[11.5px] text-warn">
                {{ $t(importError === 'timeout' ? 'holiday.importTimeout' : importError === 'empty' ?
                    'holiday.importNothing' : 'holiday.importFailed') }}
            </div>

            <!-- Import preview: nothing is written until confirmed -->
            <div v-if="importRows.length" class="rounded-xl border border-border bg-surface p-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <span class="card-label">{{ $t('holiday.importPreview') }}</span>
                    <div class="flex items-center gap-2">
                        <button type="button" class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            @click="importRows = []">
                            {{ $t('holiday.cancelBtn') }}
                        </button>
                        <button type="button" class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                            :disabled="holiday.saving" @click="applyImport">
                            {{ $t('holiday.importApply', { count: importRows.length }) }}
                        </button>
                    </div>
                </div>
                <div class="mt-2 max-h-48 overflow-y-auto space-y-1">
                    <div v-for="row in importRows" :key="row.holiday_date"
                        class="flex items-center gap-2 mono text-[11px] text-text">
                        <span class="chip shrink-0" :class="kindChip(row.holiday_kind)">{{
                            $t(kindLabel(row.holiday_kind)) }}</span>
                        <span class="w-24 shrink-0 text-text-muted">{{ row.holiday_date }}</span>
                        <span class="truncate">{{ row.holiday_name }}</span>
                    </div>
                </div>
            </div>

            <!-- List: table from md up, cards below -->
            <div class="hidden md:block overflow-auto max-h-[360px]">
                <table class="w-full">
                    <thead class="sticky top-0 z-10">
                        <tr>
                            <th class="th">{{ $t('holiday.colDate') }}</th>
                            <th class="th">{{ $t('holiday.colName') }}</th>
                            <th class="th">{{ $t('holiday.colKind') }}</th>
                            <th class="th hidden lg:table-cell">{{ $t('holiday.colSource') }}</th>
                            <th class="th !text-center">{{ $t('holiday.colCounted') }}</th>
                            <th class="th !text-center">{{ $t('common.actions') }}</th>
                        </tr>
                    </thead>
                    <tbody v-if="visible.length > 0">
                        <tr v-for="row in visible" :key="row.holiday_date" class="group even:bg-surface-raised/25">
                            <td class="td mono text-[11.5px] whitespace-nowrap">{{ row.holiday_date }}</td>
                            <td class="td font-semibold">{{ row.holiday_name }}</td>
                            <td class="td">
                                <span class="chip" :class="kindChip(row.holiday_kind)">{{
                                    $t(kindLabel(row.holiday_kind)) }}</span>
                            </td>
                            <td class="td mono text-[11px] !text-text-muted hidden lg:table-cell">
                                {{ row.holiday_source === 'import' ? $t('holiday.srcImport') : $t('holiday.srcManual')
                                }}
                            </td>
                            <td class="td !text-center">
                                <button type="button" role="switch"
                                    :aria-checked="row.holiday_included ? 'true' : 'false'"
                                    :aria-label="$t('holiday.colCounted')" :title="$t('holiday.countedHint')"
                                    class="relative w-9 h-5 rounded-full transition disabled:opacity-40"
                                    :class="row.holiday_included ? 'bg-primary' : 'bg-surface-sunken'"
                                    :disabled="holiday.saving" @click="toggleIncluded(row)">
                                    <span class="absolute top-0.5 w-4 h-4 rounded-full bg-surface shadow transition-all"
                                        :class="row.holiday_included ? 'left-4' : 'left-0.5'"></span>
                                </button>
                            </td>
                            <td class="td !text-center">
                                <div class="inline-flex items-center gap-1">
                                    <button type="button"
                                        class="w-8 h-8 rounded-lg bg-surface border border-border text-text-muted hover:bg-primary hover:text-white hover:border-primary flex items-center justify-center transition"
                                        :title="$t('common.edit')" @click="startEdit(row)">
                                        <font-awesome :icon="faPen" class="w-3 h-3" />
                                    </button>
                                    <button type="button"
                                        class="w-8 h-8 rounded-lg bg-surface border border-border text-danger hover:bg-danger hover:text-white hover:border-danger flex items-center justify-center transition"
                                        :title="$t('common.delete')" @click="remove(row)">
                                        <font-awesome :icon="faTrash" class="w-3 h-3" />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                    <tbody v-else>
                        <tr>
                            <td colspan="6">
                                <div class="state">
                                    <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                                    <p class="state-title">{{ $t('holiday.empty') }}</p>
                                    <p class="state-hint">{{ $t('holiday.emptyHint') }}</p>
                                    <button @click="openAdd"
                                        class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]">{{
                                        $t('holiday.addBtn') }}</button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <!-- Mobile cards -->
            <div class="md:hidden space-y-2 max-h-[360px] overflow-auto">
                <div v-if="visible.length === 0" class="panel">
                    <div class="state">
                        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
                        <p class="state-title">{{ $t('holiday.empty') }}</p>
                        <p class="state-hint">{{ $t('holiday.emptyHint') }}</p>
                    </div>
                </div>
                <div v-for="row in visible" :key="row.holiday_date"
                    class="rounded-xl border border-border bg-surface p-3 flex items-center gap-3">
                    <span class="w-1.5 h-10 rounded-full shrink-0"
                        :class="row.holiday_included ? 'bg-primary' : 'bg-surface-sunken'"></span>
                    <div class="flex-1 min-w-0">
                        <p class="font-semibold text-[13px] text-text truncate">{{ row.holiday_name }}</p>
                        <p class="mono text-[10.5px] text-text-muted mt-0.5">{{ row.holiday_date }} - {{
                            $t(kindLabel(row.holiday_kind)) }}</p>
                    </div>
                    <button type="button" role="switch" :aria-checked="row.holiday_included ? 'true' : 'false'"
                        :aria-label="$t('holiday.colCounted')"
                        class="relative w-9 h-5 rounded-full transition shrink-0 disabled:opacity-40"
                        :class="row.holiday_included ? 'bg-primary' : 'bg-surface-sunken'" :disabled="holiday.saving"
                        @click="toggleIncluded(row)">
                        <span class="absolute top-0.5 w-4 h-4 rounded-full bg-surface shadow transition-all"
                            :class="row.holiday_included ? 'left-4' : 'left-0.5'"></span>
                    </button>
                    <div class="flex flex-col gap-1 shrink-0">
                        <button type="button" @click="startEdit(row)" class="btn btn-primary btn-sm !px-3">{{
                            $t('common.edit') }}</button>
                        <button type="button" @click="remove(row)" class="btn btn-danger btn-sm !px-3">{{
                            $t('common.delete') }}</button>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" @ok="handleDeleteOk" @cancel="handleDeleteCancel" v-show="sm_data.isShow()" />
    <ModalHolidayForm v-model:date="editDate" v-model:name="editName" v-model:kind="editKind"
        v-model:included="editIncluded" :show="formOpen" :editing="editing" :saving="holiday.saving" :can-save="canEdit"
        :kinds="KINDS" @close="resetForm" @save="save" />
</template>
