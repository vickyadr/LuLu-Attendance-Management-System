<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useEmployee } from '~/store/employee';
import { useNotifyStore } from '~/store/notify';
import { useSchedule } from '~/store/schedule';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY, OFF_DAY_LABEL, isOffDay } from '~/utils/labels';
import { faXmark, faRotate, faLock, faTriangleExclamation, faCalendarDays } from '@fortawesome/free-solid-svg-icons';

const { t } = useI18n()

const
    auth = useAuthStore(),
    notify = useNotifyStore(),
    employee = useEmployee(),
    schedule = useSchedule(),
    shiftStore = useShiftStore(),
    config = useRuntimeConfig(),
    check = useChecker(),
    emit = defineEmits(['done']),
    validator = reactive({ first_name: '', department: '', address: '', schedule_id: '', device_uid: '' }),
    form_data = reactive({
        id: 0,
        first_name: '',
        last_name: '',
        address: '',
        department: '',
        status: 0,
        schedule_id: null,
        device_uid: null,
    }),
    isEdit = computed(() => form_data.id > 0),
    // The form lives in a dialog instead of beside the table. On a phone the
    // inline version sat below 30+ rows, so reaching it meant scrolling past the
    // whole list, and nothing on screen said a form existed. The unregistered tab
    // also needs to open it with a number already filled in, which needs a real
    // show/hide boundary rather than a panel that is always on the page.
    show = ref(false);

// Escape closes, matching ModalHolidayForm; the X sits far from the thumb on a
// phone so the keyboard shortcut matters.
function onKeydown(e) { if (e.key === 'Escape' && show.value && !submitting.value) close() }
onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))

function open() { show.value = true }
function close() { show.value = false }
const isShow = computed(() => show.value)
// Focus the first field on every open so the keyboard starts in the dialog
// instead of staying behind it on the page underneath.
watch(show, v => { if (v) nextTick(() => document.getElementById('emp-uid-input')?.focus()) })

// The submit covers both paths: a fresh record needs 'add', an existing record
// needs 'edit'. Deny-by-default means unknown users see neither button.
// Declared after `auth` exists: a computed referencing `auth` above its
// declaration throws on first access, not at definition time.
const canAdd = computed(() => auth.hasActionAccess('employee', 'can_add'))
const canEdit = computed(() => auth.hasActionAccess('employee', 'can_edit'))
// form_data.id > 0 means the form is editing an existing record; 0 means new.
const canSubmit = computed(() => (form_data.id > 0 ? canEdit.value : canAdd.value))

const submitting = ref(false)

// schedule options: only parent schedules
const scheduleOptions = computed(() => {
    const all = schedule.contents || []
    const parents = all.filter(s => s.parent === s.id)
    return parents
})
const shiftMap = computed(() => {
    const m = {}
    for (const s of (shiftStore.contents || [])) m[s.id] = s
    return m
})
function scheduleBadge(s) {
    const t = s.type
    if (t === 1) return $t('report.periodDaily')
    if (t === 7) return $t('report.periodWeekly')
    if (t === 14) return $t('schedule.weeks2')
    if (t === 21) return $t('schedule.weeks3')
    if (t === 28) return $t('report.periodMonthly')
    return `T${t}`
}
function scheduleDesc(s) {
    // count doms for this parent
    const all = schedule.contents || []
    const cnt = all.filter(x => x.parent === s.parent).length
    return t('common.daysCount', { n: cnt })
}
function previewSchedule(s) {
    const all = (schedule.contents || []).filter(x => x.parent === s.parent).sort((a, b) => a.dom - b.dom)
    // map shift id to name
    return all.map(x => {
        const name = x.shift_name || (shiftMap.value[x.schedule_shift_id]?.name) || (x.schedule_shift_id === 0 ? OFF_DAY : '?')
        const off = x.schedule_shift_id === 0 || isOffDay(name)
        return { dom: x.dom, name: off ? OFF_DAY_LABEL : name, start: x.start, end: x.end, isLibur: off }
    })
}

async function ensureSchedules() {
    if (schedule.contents && schedule.contents.length) return
    try {
        const r = await apiFetch(`${config.public.apiBase}/schedule/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) schedule.set(r.data)
    } catch (e) { console.error(e) }
}
async function ensureShifts() {
    if (shiftStore.contents && shiftStore.contents.length) return
    try {
        const r = await apiFetch(`${config.public.apiBase}/shift/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) shiftStore.set(r.data)
    } catch (e) { console.error(e) }
}
onMounted(() => { ensureSchedules(); ensureShifts() })

// Department suggestions come from the departments already present in the loaded
// employee list, so no extra request is needed and a fresh value typed by hand still
// works. Same Set/sort shape employee.vue and report.vue use for their filter.
const deptOpen = ref(false)
const deptIdx = ref(-1)
const deptOptions = computed(() => {
    const set = new Set((employee.contents || []).map(x => (x.department || '').trim()).filter(Boolean))
    const all = [...set].sort((a, b) => a.localeCompare(b))
    const q = form_data.department.trim().toLowerCase()
    const matched = q ? all.filter(d => d.toLowerCase().includes(q)) : all
    // An exact match has nothing left to pick, so keep it out of the list.
    return matched.filter(d => d !== form_data.department.trim())
})
const deptVisible = computed(() => deptOpen.value && deptOptions.value.length > 0)

// The list only exists while there is a query to match against, so an empty
// field never shows a full dump under the cursor. ArrowDown keeps working as a
// way to open the list that typing already revealed.
function deptTyped() {
    deptIdx.value = -1
    deptOpen.value = form_data.department.trim().length > 0
}
function deptPick(d) {
    form_data.department = d
    deptOpen.value = false
    deptIdx.value = -1
}
function deptHide() { deptOpen.value = false; deptIdx.value = -1 }
function deptKey(e) {
    const n = deptOptions.value.length
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        // preventDefault: arrows would otherwise scroll the page behind the list.
        e.preventDefault()
        if (!deptVisible.value) {
            // Reopen only when there is a query; an empty field never lists.
            if (form_data.department.trim().length > 0) { deptOpen.value = true; deptIdx.value = -1 }
            return
        }
        if (!n) return
        // Clamp ArrowUp at 0 so one press lands on the first row.
        deptIdx.value = e.key === 'ArrowDown' ? Math.min(deptIdx.value + 1, n - 1) : Math.max(deptIdx.value - 1, 0)
        nextTick(() => document.querySelector('.dept-item.is-active')?.scrollIntoView({ block: 'nearest' }))
    } else if (e.key === 'Enter') {
        // Only swallow Enter when a suggestion is highlighted, otherwise the form
        // must still submit normally.
        if (deptVisible.value && deptIdx.value >= 0) {
            e.preventDefault()
            deptPick(deptOptions.value[deptIdx.value])
        }
    } else if (e.key === 'Escape') {
        deptHide()
    }
}

// Digits only. type="number" would still accept e, +, -, . and silently return
// null for them; stripping here means the model never holds anything the column
// or the uniqueness check could reject in a confusing way.
function uidInput(e) {
    const cleaned = e.target.value.replace(/\D/g, '')
    if (e.target.value !== cleaned) e.target.value = cleaned
    form_data.device_uid = cleaned === '' ? null : Number(cleaned)
}

function validate() {
    validator.first_name = ''; validator.department = ''; validator.address = ''; validator.schedule_id = ''; validator.device_uid = ''
    if (!form_data.first_name || form_data.first_name.trim().length < 3) validator.first_name = t('employee.errNameShort')
    if (!form_data.department || form_data.department.trim().length < 3) validator.department = t('employee.errDeptShort')
    if (form_data.address && form_data.address.length >= 160) validator.address = t('employee.errMaxChars')
    if (form_data.schedule_id === null || form_data.schedule_id === '') validator.schedule_id = t('employee.pickSchedule')
    // Required so every employee always owns exactly one machine number: an
    // unlinked employee means their punches silently land in the unregistered
    // list, which is the exact failure this field exists to prevent.
    if (form_data.device_uid === null || form_data.device_uid === '' || Number(form_data.device_uid) < 1) validator.device_uid = t('employee.errUidRequired')
    return !validator.first_name && !validator.department && !validator.address && !validator.schedule_id && !validator.device_uid
}

function fillForm(data) {
    // data from ListEmployee: id, first_name, last_name, department, address, status, schedule_id, device_uid
    form_data.id = data.id || data.employee_id || 0
    form_data.first_name = data.first_name || data.employee_fname || ''
    form_data.last_name = data.last_name || data.employee_lname || ''
    form_data.address = data.address || data.employee_address || ''
    form_data.department = data.department || data.employee_department || ''
    form_data.status = data.status ?? data.employee_status ?? 0
    form_data.schedule_id = data.schedule_id ?? data.employee_schedule_id ?? null
    form_data.device_uid = data.device_uid ?? data.employee_device_uid ?? null
    // ensure schedules loaded
    ensureSchedules()
}

function resetForm() {
    form_data.id = 0; form_data.first_name = ''; form_data.last_name = ''; form_data.address = ''; form_data.department = ''; form_data.status = 0; form_data.schedule_id = null; form_data.device_uid = null
    validator.first_name = ''; validator.department = ''; validator.address = ''; validator.schedule_id = ''; validator.device_uid = ''
    deptHide()
}


async function submit() {
    if (!validate()) return
    const isEditMode = form_data.id > 0
    // Belt and braces: the submit button is hidden without permission, but form
    // submit (Enter key) can still fire, and the server would answer 403 anyway.
    if (isEditMode ? !canEdit.value : !canAdd.value) {
        notify.warning(t('common.noAccess'), t('device.noPermission'))
        return
    }
    submitting.value = true
    const url = isEditMode ? `${config.public.apiBase}/employee/edit` : `${config.public.apiBase}/employee/add`
    const body = {
        id: isEditMode ? form_data.id : undefined,
        first_name: form_data.first_name.trim(),
        // employee_lname is NOT NULL in the schema, so an empty last name must go
        // as an empty string. Sending null turned an optional field into a 500.
        last_name: form_data.last_name?.trim() || '',
        department: form_data.department.trim(),
        address: form_data.address?.trim() || '',
        status: Number(form_data.status) || 0,
        schedule_id: Number(form_data.schedule_id),
        device_uid: form_data.device_uid ? Number(form_data.device_uid) : undefined,
    }
    try {
        const response = await apiFetch(url, { method: 'POST', body, headers: auth.confHeaders() })
        if (response.code === 200) {
            // reload employee list
            try {
                const r2 = await apiFetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() })
                if (r2.code === 200) employee.set(r2.data)
            } catch { }
            const msg = response.message || (isEditMode ? t('employee.updated') : t('employee.created'))
            notify.success(isEditMode ? t('employee.editTitle') : t('employee.addTitle'), msg)
            // Both paths emit: adding an employee with a device uid pulls that uid
            // out of the unregistered list, and the add branch is the one the
            // Register button uses, so an edit-only emit leaves that row on screen.
            resetForm()
            emit('done')
            // Successful save ends the interaction either way; leaving the dialog
            // open after an add would let the next submit post the same id twice.
            close()
        } else {
            if (response.data) {
                validator.first_name = response.data.first_name || ''
                validator.department = response.data.department || ''
                validator.address = response.data.address || response.data.location || ''
                validator.schedule_id = response.data.schedule_id || ''
            }
            notify.error(t('common.saveFailed'), response.message || t('employee.saveFailed'))
        }
    } catch (e) {
        const msg = e?.data?.message || e?.message || 'Connection failed'
        if (e?.data?.data) {
            validator.first_name = e.data.data.first_name || ''
            validator.department = e.data.data.department || ''
        }
        notify.error(t('common.connFailed'), msg)
    } finally { submitting.value = false }
}

async function del_employee(id) {
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/delete/${id}`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            employee.removeList(id)
            // also reload to sync
            try {
                const r2 = await apiFetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() })
                if (r2.code === 200) employee.set(r2.data)
            } catch { }
        }
        notify[r.code === 200 ? 'success' : 'error'](t('employee.deleteTitle'), r.message || t('common.done'))
    } catch (e) {
        notify.error(t('common.deleteFailed'), e?.data?.message || e?.message)
    }
}

// open() is what the unregistered tab calls to start a new employee with that
// machine number already filled in; fillForm stays the edit path.
function openNew(presetUid) {
    resetForm()
    if (presetUid != null) form_data.device_uid = Number(presetUid)
    open()
}

defineExpose({ fillForm, del_employee, resetForm, open, close, isShow, openNew })
</script>

<template>
    <!--
        v-if, not v-show: a hidden fixed overlay keeps swallowing clicks on the
        page underneath, because Tailwind's `fixed` wins over v-show's `display:none`.
    -->
    <div v-if="show" class="fixed inset-0 z-[100] grid place-items-center bg-sidebar-top/55 backdrop-blur-sm p-3 sm:p-4"
        @click.self="!submitting && close()">
        <div class="panel w-full max-w-2xl max-h-[92dvh] flex flex-col overflow-hidden shadow-tint-lg" role="dialog"
            aria-modal="true">

            <!-- dialog head -->
            <div class="panel-head shrink-0">
                <span class="inline-flex items-center gap-2 min-w-0">
                    <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                    <span class="truncate">{{ isEdit ? $t('employee.editTitle') : $t('employee.addTitle') }}</span>
                </span>
                <div class="flex items-center gap-2 shrink-0">
                    <span class="chip" :class="isEdit ? 'chip-warn' : 'chip-ok'">{{ isEdit ? $t('employee.editMode') :
                        $t('employee.createMode') }}</span>
                    <button type="button" @click="close"
                        class="w-11 h-11 grid place-content-center rounded-xl border border-border bg-surface text-text-muted hover:bg-surface-raised transition"
                        :aria-label="$t('common.cancel')" :disabled="submitting">
                        <font-awesome :icon="faXmark" class="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>

            <form @submit.prevent="submit" class="p-4 sm:p-5 space-y-4 overflow-y-auto min-h-0">

                <!-- identity -->
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-3.5">
                    <div>
                        <label for="emp-uid-input" class="label">{{ $t('employee.deviceUid') }} <span
                                class="text-danger">*</span></label>
                        <div class="relative">
                            <input :value="form_data.device_uid ?? ''" id="emp-uid-input" type="text"
                                inputmode="numeric" pattern="[0-9]*" maxlength="10" required
                                :placeholder="$t('employee.deviceUidPh')" :disabled="isEdit" @input="uidInput"
                                class="field mono has-icon-right"
                                :class="isEdit ? '!bg-warn-bg !border-amber-200 !text-cat-amber cursor-not-allowed' : ''">
                            <span v-if="isEdit"
                                class="absolute right-2 top-1/2 -translate-y-1/2 w-6 h-6 rounded-md bg-amber-500 text-white grid place-content-center"
                                :title="$t('employee.deviceUidLocked')">
                                <font-awesome :icon="faLock" class="w-3 h-3" />
                            </span>
                        </div>
                        <p v-if="validator.device_uid" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">
                            {{ validator.device_uid }}</p>
                        <p v-else-if="isEdit" class="mt-1 mono text-[10px] text-cat-amber">{{
                            $t('employee.deviceUidLocked') }}</p>
                    </div>

                    <div>
                        <label for="emp-first-name" class="label">{{ $t('employee.firstName') }} <span
                                class="text-danger">*</span></label>
                        <input id="emp-first-name" v-model="form_data.first_name" type="text" required
                            :placeholder="$t('employee.firstNamePh')" class="field">
                        <p v-if="validator.first_name" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">
                            {{ validator.first_name }}</p>
                    </div>

                    <div>
                        <label for="emp-last-name" class="label">{{ $t('employee.lastName') }}</label>
                        <input id="emp-last-name" v-model="form_data.last_name" type="text"
                            :placeholder="$t('employee.lastNamePh')" class="field">
                    </div>

                    <div>
                        <label for="emp-department" class="label">{{ $t('employee.department') }} <span
                                class="text-danger">*</span></label>
                        <div class="relative">
                            <input id="emp-department" v-model="form_data.department" type="text" required
                                :placeholder="$t('employee.deptPh')" class="field" @blur="deptHide" @input="deptTyped"
                                @keydown="deptKey">
                            <!-- Suggestions from existing departments. Absolute so it never
                                 shifts the grid; mousedown.prevent keeps the click alive,
                                 since a blur-closing list unmounts before click fires. -->
                            <div v-if="deptVisible"
                                class="absolute z-30 left-0 right-0 top-full mt-1 max-h-56 overflow-auto rounded-xl border border-border bg-surface shadow-tint-md">
                                <button v-for="(d, i) in deptOptions" :key="d" type="button" tabindex="-1"
                                    class="dept-item w-full text-left px-3.5 min-h-[44px] text-[13px] text-text transition-colors"
                                    :class="{ 'is-active': i === deptIdx }" @mousedown.prevent="deptPick(d)">
                                    <span class="truncate">{{ d }}</span>
                                </button>
                            </div>
                        </div>
                        <p v-if="validator.department" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">
                            {{ validator.department }}</p>
                        <p v-else-if="deptVisible" class="mt-1 mono text-[10px] text-text-muted">↑↓ to pick • ENTER to
                            use • ESC to close</p>
                    </div>

                    <div>
                        <label for="emp-status" class="label">{{ $t('employee.status') }}</label>
                        <select id="emp-status" v-model.number="form_data.status" class="field">
                            <option :value="0">{{ $t('common.active') }}</option>
                            <option :value="1">{{ $t('employee.statusInactive') }}</option>
                        </select>
                    </div>

                    <div class="sm:col-span-2">
                        <label for="emp-address" class="label">{{ $t('employee.address') }}</label>
                        <input id="emp-address" v-model="form_data.address" type="text"
                            :placeholder="$t('employee.addressPh')" class="field">
                        <p v-if="validator.address" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
                            validator.address }}</p>
                    </div>
                </div>

                <!-- schedule assignment -->
                <section class="rounded-xl border border-border bg-surface-raised/50 p-3.5 space-y-3">
                    <p class="flex items-center gap-2 card-label !text-primary-deeper">
                        <font-awesome :icon="faCalendarDays" class="w-3.5 h-3.5" />
                        {{ $t('employee.scheduleAssignment') }} <span class="text-danger">*</span>
                    </p>

                    <select v-model="form_data.schedule_id" class="field" :aria-label="$t('employee.workSchedule')">
                        <option :value="null" disabled>- {{ $t('employee.pickSchedule') }} -</option>
                        <option v-for="s in scheduleOptions" :key="s.parent" :value="s.parent">{{ s.name }} - {{
                            scheduleBadge(s) }} • {{ scheduleDesc(s) }}</option>
                    </select>

                    <p v-if="validator.schedule_id" role="alert" class="text-[11.5px] font-medium text-danger">{{
                        validator.schedule_id }}</p>
                    <p v-else-if="scheduleOptions.length === 0"
                        class="flex items-start gap-2 rounded-lg border border-cat-amber/25 bg-warn-bg px-3 py-2.5 text-[11.5px] leading-relaxed text-cat-amber">
                        <font-awesome :icon="faTriangleExclamation" class="w-3.5 h-3.5 mt-0.5 shrink-0" />
                        <span>{{ $t('employee.emptySchedulesFirst') }}</span>
                    </p>

                    <!-- preview selected schedule -->
                    <div v-if="form_data.schedule_id"
                        class="rounded-lg bg-surface border border-border px-3 py-2.5 space-y-2">
                        <template
                            v-for="s in scheduleOptions.filter(x => String(x.parent) === String(form_data.schedule_id))"
                            :key="s.parent">
                            <p class="mono text-[10px] font-bold tracking-[0.12em] text-text-muted uppercase">{{
                                $t('employee.preview') }}: {{ s.name }} - {{ scheduleBadge(s) }} • {{ scheduleDesc(s) }}
                            </p>
                            <div class="flex flex-wrap gap-1.5">
                                <span v-for="p in previewSchedule(s)" :key="p.dom"
                                    class="inline-flex items-center gap-1 px-2 py-1 rounded-full border text-[10.5px] mono"
                                    :class="p.isLibur ? 'bg-surface-raised border-border text-text-muted' : 'bg-surface border-border-strong text-text'">
                                    <span class="w-1.5 h-1.5 rounded-full"
                                        :class="p.isLibur ? 'bg-text-muted/40' : 'bg-primary'"></span>
                                    D{{ p.dom }}: {{ p.name }} <span v-if="!p.isLibur && p.start != null"
                                        class="opacity-60">{{ Math.floor(p.start / 3600).toString().padStart(2, '0') }}:{{
                                            Math.floor((p.start %3600)/60).toString().padStart(2,'0') }}</span>
                                </span>
                            </div>
                        </template>
                    </div>
                </section>

                <div class="flex flex-col sm:flex-row gap-2 pt-1">
                    <button v-if="canSubmit" type="submit" :disabled="submitting"
                        class="btn btn-primary flex-1 sm:flex-none sm:!px-7">
                        <font-awesome v-if="submitting" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
                        {{ submitting ? $t('common.saving') : (isEdit ? $t('employee.updateEmployee') :
                            $t('employee.saveEmployee')) }}
                    </button>
                    <button type="button" @click="resetForm" class="btn btn-secondary flex-1 sm:flex-none">{{
                        $t('common.reset')
                        }}</button>
                    <button v-if="isEdit" type="button" @click="resetForm(); close(); emit('done')"
                        class="btn btn-quiet flex-1 sm:flex-none">{{ $t('common.cancelEdit') }}</button>
                </div>
            </form>
        </div>
    </div>
</template>

<style scoped>
/* Highlight driven by deptIdx. Without this rule the class renders but looks
   identical to an unselected row. */
.dept-item:hover,
.dept-item.is-active {
    background: var(--surface-raised);
    color: var(--primary-deeper);
    font-weight: 600;
}
</style>
