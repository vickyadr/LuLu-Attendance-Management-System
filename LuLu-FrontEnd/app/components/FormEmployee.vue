<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useEmployee } from '~/store/employee';
import { useSchedule } from '~/store/schedule';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY, OFF_DAY_LABEL, isOffDay } from '~/utils/labels';

const
    auth = useAuthStore(),
    employee = useEmployee(),
    schedule = useSchedule(),
    shiftStore = useShiftStore(),
    config = useRuntimeConfig(),
    check = useChecker(),
    emit = defineEmits(['notif', 'done']),
    validator = reactive({ first_name: '', departement: '', address: '', schedule_id: '' }),
    form_data = reactive({
        id: 0,
        first_name: '',
        last_name: '',
        address: '',
        departement: '',
        status: 0,
        schedule_id: null,
    }),
    isEdit = computed(() => form_data.id > 0);

const submitting = ref(false)

// schedule options: only parrent schedules
const scheduleOptions = computed(() => {
    const all = schedule.contents || []
    const parents = all.filter(s => s.parrent === s.id)
    return parents
})
const shiftMap = computed(() => {
    const m = {}
    for (const s of (shiftStore.contents || [])) m[s.id] = s
    return m
})
function scheduleBadge(s) {
    const t = s.type
    if (t === 1) return 'DAILY'
    if (t === 7) return 'WEEKLY'
    if (t === 14) return '2 WEEKS'
    if (t === 21) return '3 WEEKS'
    if (t === 28) return 'MONTHLY'
    return `T${t}`
}
function scheduleDesc(s) {
    // count doms for this parrent
    const all = schedule.contents || []
    const cnt = all.filter(x => x.parrent === s.parrent).length
    return `${cnt} days • type ${s.type}`
}
function previewSchedule(s) {
    const all = (schedule.contents || []).filter(x => x.parrent === s.parrent).sort((a, b) => a.dom - b.dom)
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
        const r = await $fetch(`${config.public.apiBase}/schedule/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) schedule.set(r.data)
    } catch (e) { console.error(e) }
}
async function ensureShifts() {
    if (shiftStore.contents && shiftStore.contents.length) return
    try {
        const r = await $fetch(`${config.public.apiBase}/shift/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) shiftStore.set(r.data)
    } catch (e) { console.error(e) }
}
onMounted(() => { ensureSchedules(); ensureShifts() })

function validate() {
    validator.first_name = ''; validator.departement = ''; validator.address = ''; validator.schedule_id = ''
    if (!form_data.first_name || form_data.first_name.trim().length < 3) validator.first_name = 'First name must be at least 3 characters'
    if (!form_data.departement || form_data.departement.trim().length < 3) validator.departement = 'Department must be at least 3 characters'
    if (form_data.address && form_data.address.length >= 160) validator.address = 'Max 160 characters'
    if (form_data.schedule_id === null || form_data.schedule_id === '') validator.schedule_id = 'Pick a schedule'
    return !validator.first_name && !validator.departement && !validator.address && !validator.schedule_id
}

function fillForm(data) {
    // data from ListEmployee: id, first_name, last_name, departement, address, status, schedule_id
    form_data.id = data.id || data.employee_id || 0
    form_data.first_name = data.first_name || data.employee_fname || ''
    form_data.last_name = data.last_name || data.employee_lname || ''
    form_data.address = data.address || data.employee_address || ''
    form_data.departement = data.departement || data.employee_departement || ''
    form_data.status = data.status ?? data.employee_status ?? 0
    form_data.schedule_id = data.schedule_id ?? data.employee_schedule_id ?? null
    // ensure schedules loaded
    ensureSchedules()
}

function resetForm() {
    form_data.id = 0; form_data.first_name = ''; form_data.last_name = ''; form_data.address = ''; form_data.departement = ''; form_data.status = 0; form_data.schedule_id = null
    validator.first_name = ''; validator.departement = ''; validator.address = ''; validator.schedule_id = ''
}

function callbackNotif(d) { emit('notif', d) }

async function submit() {
    if (!validate()) return
    submitting.value = true
    const isEditMode = form_data.id > 0
    const url = isEditMode ? `${config.public.apiBase}/employee/edit` : `${config.public.apiBase}/employee/add`
    const body = {
        id: isEditMode ? form_data.id : undefined,
        first_name: form_data.first_name.trim(),
        last_name: form_data.last_name?.trim() || null,
        departement: form_data.departement.trim(),
        address: form_data.address?.trim() || '',
        status: Number(form_data.status) || 0,
        schedule_id: Number(form_data.schedule_id),
    }
    try {
        const response = await $fetch(url, { method: 'POST', body, headers: auth.confHeaders() })
        if (response.code === 200) {
            // reload employee list
            try {
                const r2 = await $fetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() })
                if (r2.code === 200) employee.set(r2.data)
            } catch { }
            const msg = response.message || (isEditMode ? 'Employee updated' : 'Employee added')
            callbackNotif({ title: isEditMode ? 'Edit Employee' : 'Add Employee', message: msg })
            if (!isEditMode) resetForm()
            else { resetForm(); emit('done') }
        } else {
            if (response.data) {
                validator.first_name = response.data.first_name || ''
                validator.departement = response.data.departement || ''
                validator.address = response.data.address || response.data.location || ''
                validator.schedule_id = response.data.schedule_id || ''
            }
            callbackNotif({ title: 'Gagal', message: response.message || 'Failed to save employee' })
        }
    } catch (e) {
        const msg = e?.data?.message || e?.message || 'Connection failed'
        if (e?.data?.data) {
            validator.first_name = e.data.data.first_name || ''
            validator.departement = e.data.data.departement || ''
        }
        callbackNotif({ title: 'Error', message: msg })
    } finally { submitting.value = false }
}

async function del_employee(id) {
    try {
        const r = await $fetch(`${config.public.apiBase}/employee/delete/${id}`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            employee.removeList(id)
            // also reload to sync
            try {
                const r2 = await $fetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() })
                if (r2.code === 200) employee.set(r2.data)
            } catch { }
        }
        callbackNotif({ title: 'Delete Employee', message: r.message || 'Selesai' })
    } catch (e) {
        callbackNotif({ title: 'Error', message: e?.data?.message || e?.message })
    }
}

defineExpose({ fillForm, del_employee, resetForm })
</script>

<template>
    <div class="text-left">
        <div
            class="px-5 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-lime-50/40 flex items-center justify-between tech-grid-soft">
            <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                <h3 class="text-sm font-bold tracking-wide text-green-900 uppercase mono">{{ isEdit ? 'Edit Employee' :
                    'Add Employee' }}</h3>
                <span class="chip chip-emerald mono !text-[9px] hidden sm:inline-flex">SCHEDULE • CUSTOM SHIFT</span>
            </div>
            <span class="chip mono !text-[9px]" :class="isEdit ? 'bg-amber-500 text-white' : 'chip-emerald'">{{ isEdit ?
                'EDIT MODE' : 'CREATE' }}</span>
        </div>

        <form @submit.prevent="submit" class="px-5 py-4 space-y-4">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                <div>
                    <label
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">First
                        Name <span class="text-red-500">*</span></label>
                    <input v-model="form_data.first_name" type="text" required placeholder="e.g. Budi"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30">
                    <p v-if="validator.first_name" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{
                        validator.first_name }}</p>
                </div>
                <div>
                    <label
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Last
                        Name</label>
                    <input v-model="form_data.last_name" type="text" placeholder="e.g. Santoso"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30">
                </div>
                <div>
                    <label
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Department
                        <span class="text-red-500">*</span></label>
                    <input v-model="form_data.departement" type="text" required
                        placeholder="e.g. Warehouse A, Production"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30">
                    <p v-if="validator.departement" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{
                        validator.departement }}</p>
                </div>
                <div>
                    <label
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Status</label>
                    <select v-model.number="form_data.status"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option :value="0">Active</option>
                        <option :value="1">Inactive</option>
                    </select>
                </div>
                <div class="lg:col-span-2">
                    <label
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Address</label>
                    <input v-model="form_data.address" type="text" placeholder="12 Example St. (optional, max 160)"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30">
                    <p v-if="validator.address" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{
                        validator.address }}</p>
                </div>
            </div>

            <!-- Schedule picker -->
            <div class="hud-frame glass rounded-2xl border border-emerald-100 overflow-hidden">
                <div
                    class="px-4 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-white flex items-center justify-between">
                    <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700">WORK SCHEDULE <span
                            class="text-red-500">*</span></span>
                </div>
                <div class="p-3.5 space-y-3">
                    <select v-model="form_data.schedule_id"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option :value="null" disabled>— Pick a schedule —</option>
                        <option v-for="s in scheduleOptions" :key="s.parrent" :value="s.parrent">{{ s.name }} — {{
                            scheduleBadge(s) }} • {{ scheduleDesc(s) }}</option>
                    </select>
                    <p v-if="validator.schedule_id" class="text-xs font-medium text-red-500 ml-1">{{
                        validator.schedule_id }}</p>
                    <p v-else-if="scheduleOptions.length === 0"
                        class="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-xl px-3 py-2.5">No
                        schedules yet — create one first under Shift → Schedule tab. Patterns: Daily (1), Weekly (7),
                        2–4 weeks (14/21/28), with day off per day.</p>

                    <!-- Preview selected schedule -->
                    <div v-if="form_data.schedule_id"
                        class="rounded-xl bg-emerald-50/60 border border-emerald-100 px-3 py-2.5 space-y-2">
                        <template
                            v-for="s in scheduleOptions.filter(x => String(x.parrent) === String(form_data.schedule_id))"
                            :key="s.parrent">
                            <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700">PREVIEW: {{ s.name
                                }} — {{ scheduleBadge(s) }} • {{ scheduleDesc(s) }}</p>
                            <div class="flex flex-wrap gap-1.5">
                                <span v-for="p in previewSchedule(s)" :key="p.dom"
                                    class="inline-flex items-center gap-1 px-2 py-1 rounded-full border text-[11px] mono"
                                    :class="p.isLibur ? 'bg-slate-50 border-slate-200 text-slate-600' : 'bg-white border-emerald-200 text-emerald-700'">
                                    <span class="w-1.5 h-1.5 rounded-full"
                                        :class="p.isLibur ? 'bg-slate-400' : 'bg-emerald-500'"></span>
                                    D{{ p.dom }}: {{ p.name }} <span v-if="!p.isLibur && p.start != null"
                                        class="opacity-60">{{ Math.floor(p.start / 3600).toString().padStart(2, '0') }}:{{
                                            Math.floor((p.start %3600)/60).toString().padStart(2,'0') }}</span>
                                </span>
                            </div>
                        </template>
                    </div>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-3 pt-1">
                <button type="submit" :disabled="submitting"
                    class="btn-emerald rounded-xl px-8 h-11 text-sm font-bold mono tracking-widest disabled:opacity-60 inline-flex items-center gap-2">
                    <svg v-if="submitting" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4">
                        </circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z">
                        </path>
                    </svg>
                    {{ submitting ? 'Menyimpan...' : (isEdit ? 'Update Employee' : 'Save Employee') }}
                </button>
                <button type="button" @click="resetForm"
                    class="rounded-xl px-8 h-11 text-sm font-medium bg-white border-2 border-emerald-100 text-emerald-700 hover:bg-emerald-50">Reset</button>
                <button v-if="isEdit" type="button" @click="resetForm(); emit('done')"
                    class="rounded-xl px-6 h-11 text-sm font-medium bg-slate-50 border border-slate-200 text-slate-600 hover:bg-slate-100">Cancel
                    Edit</button>
            </div>
        </form>
    </div>
</template>
