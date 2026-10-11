<script setup>
import ListEmployee from '~/components/ListEmployee.vue';
import FormEmployee from '~/components/FormEmployee.vue';
import ListUnregistered from '~/components/ListUnregistered.vue';
import { useEmployee } from '~/store/employee';
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';
import { SMData } from '~/components/SimpleModal.vue';

definePageMeta({ middleware: ["get-auth"], layout: 'default' });
useHead({ title: "LuLu - Employees & Schedules" });

const { t } = useI18n()
const compEmployee = ref()
const listRef = ref()
const unregRef = ref()
const employeeStore = useEmployee()
const notify = useNotifyStore()
const sm_data = new SMData()

const search = ref('')
const deptFilter = ref('')
const statusFilter = ref('')
// Registered employees and unregistered device users are the same data seen from
// two sides, so they share the page as tabs instead of stacking one below the
// other and forcing a scroll hunt.
const activeTab = ref('registered')
// Count for the tab badge. Fetched once on mount so the tab tells the operator
// there is something to look at before they click; the list itself loads when
// the tab opens.
const unregCount = ref(0)
// Pinia stores must be resolved in setup context, not inside the async callback.
const auth = useAuthStore()
const config = useRuntimeConfig()
// Declared after `auth`: a computed reading it above its declaration throws on
// first access, not at definition time. Deny-by-default, same rule as the form.
const canAdd = computed(() => auth.hasActionAccess('employee', 'can_add'))

async function loadUnregCount() {
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/unregistered`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) unregCount.value = (r.data || []).length
    } catch { /* badge is cosmetic; leave it at 0 */ }
}

const depts = computed(() => {
    const s = new Set((employeeStore.contents || []).map(x => x.departement).filter(Boolean))
    return [...s].sort()
})
const total = computed(() => (employeeStore.contents || []).length)
const aktif = computed(() => (employeeStore.contents || []).filter(x => String(x.status) === '0').length)
const nonaktif = computed(() => total.value - aktif.value)

// Import reports per-row failures as a list, not a single message: the operator
// needs the row number to fix the file, and a toast that truncates would hide
// which lines to look at.
function cbImport(res) {
    const d = res?.data?.[0] || res?.data || {}
    const errs = d.errors || []
    if (res?.code === 200 && errs.length === 0) {
        notify.success(t('employee.importTitle'), `${d.inserted || 0} imported`)
    } else if (res?.code === 200) {
        const head = errs.slice(0, 5).map(e => `${t('employee.importRow', { n: e.row })}: ${e.field} ${e.reason}`).join('\n')
        const more = errs.length > 5 ? `\n+${errs.length - 5} more` : ''
        notify.warning(t('employee.importTitle'), `${d.inserted || 0} imported, ${errs.length} failed\n${head}${more}`, 0)
    } else {
        notify.error(t('employee.importTitle'), res?.message || 'Import failed')
    }
}

function cbEdit(id) {
    const all = employeeStore.contents || []
    const found = all.find(x => String(x.id) === String(id) || String(x.employee_id) === String(id))
    if (found) { compEmployee.value?.fillForm(found); compEmployee.value?.open() }
    else showModal({ title: 'Edit', message: 'Employee not found' })
}
// A row in the unregistered list is a person missing from the employee table, so
// its number is exactly what the new-employee form needs: no looking it up, no
// retyping it.
function cbLink(uid) {
    compEmployee.value?.openNew(uid)
}
function cbNew() {
    compEmployee.value?.resetForm()
    compEmployee.value?.open()
}
function cbDelete(id) {
    sm_data.setText("Delete Employee", `Employee #${id} will be deleted. Continue?`)
    sm_data.setHelper("del-employee", { id })
    sm_data.showOKCancel()
}
async function handleOk() {
    const helper = sm_data.getHelper("del-employee")
    if (helper) await compEmployee.value?.del_employee(helper.id)
    // Deleting an employee orphans its punches, so refresh the unregistered list.
    unregRef.value?.reload()
    loadUnregCount()
    sm_data.clear()
}
function handleCancel() { sm_data.clear() }
function clearFilters() { search.value = ''; deptFilter.value = ''; statusFilter.value = '' }

const filteredCount = computed(() => {
    let arr = employeeStore.contents || []
    const q = (search.value || '').trim().toLowerCase()
    if (q) arr = arr.filter(x => `${x.first_name || ''} ${x.last_name || ''} ${x.departement || ''}`.toLowerCase().includes(q))
    if (deptFilter.value) arr = arr.filter(x => (x.departement || '') === deptFilter.value)
    if (statusFilter.value !== '') arr = arr.filter(x => String(x.status) === String(statusFilter.value))
    return arr.length
})
onMounted(() => loadUnregCount())
</script>

<template>
    <div class="space-y-4">
        <!-- Header -->
        <div
            class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col lg:flex-row lg:items-center gap-4 tech-grid-soft">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3">
                <div
                    class="flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="1.8">
                        <path d="M16 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2" />
                        <circle cx="9" cy="7" r="4" />
                        <path d="M22 11a3 3 0 00-3-3 3 3 0 00-3 3 3 3 0 003 3 3 3 0 003-3z" />
                        <path d="M15 11h3" />
                        <path d="M16.5 9.5v3" />
                    </svg>
                </div>
                <div>
                    <h1 class="display text-base sm:text-lg font-extrabold text-green-900 tracking-tight">{{
                        $t('employee.listTitle') }}</h1>
                    <div class="mt-2 flex flex-wrap items-center gap-2 mono text-[10px]">
                        <span class="chip chip-emerald">{{ total }} {{ $t('employee.employeesUnit') }}</span>
                        <span class="inline-flex items-center gap-1.5 text-emerald-700/40"><span
                                class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> {{
                                    $t('employee.statActiveInactive', { active: aktif, inactive: nonaktif }) }}</span>
                    </div>
                </div>
            </div>
            <div class="lg:ml-auto flex items-center gap-2">
                <div
                    class="hidden sm:flex items-center gap-2 px-3 py-2 rounded-xl bg-white border border-emerald-100 mono text-[10px]">
                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span> {{ $t('employee.statRegistered', {
                        total:
                    total }) }}
                    <span class="w-px h-3 bg-emerald-100"></span>
                    <span class="text-emerald-700">{{ aktif }} {{ $t('common.active').toUpperCase() }}</span>
                </div>
                <button v-if="canAdd" @click="cbNew()"
                    class="btn-emerald rounded-xl px-5 h-11 text-sm font-bold mono tracking-widest inline-flex items-center gap-2 shadow-sm">
                    <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="2.5">
                        <path d="M12 5v14M5 12h14" />
                    </svg>
                    {{ $t('employee.addTitle') }}
                </button>
            </div>
        </div>

        <!-- Tabs: registered employees vs unregistered user ids -->
        <div class="flex items-center gap-1.5 px-1">
            <button @click="activeTab = 'registered'"
                class="px-4 h-9 rounded-t-xl mono text-[11px] font-bold transition flex items-center gap-1.5 border-b-2"
                :class="activeTab === 'registered'
                    ? 'bg-white border-emerald-600 text-emerald-700'
                    : 'bg-emerald-50/40 border-transparent text-emerald-700/50 hover:text-emerald-700 hover:bg-emerald-50'">
                {{ $t('employee.listTitle') }}
                <span class="px-1.5 py-0.5 rounded-full bg-emerald-100 text-emerald-700 text-[10px]">{{ total }}</span>
            </button>
            <button @click="activeTab = 'unregistered'"
                class="px-4 h-9 rounded-t-xl mono text-[11px] font-bold transition flex items-center gap-1.5 border-b-2"
                :class="activeTab === 'unregistered'
                    ? 'bg-white border-amber-500 text-amber-800'
                    : 'bg-amber-50 border-transparent text-amber-800 hover:text-amber-900 hover:bg-amber-100'">
                {{ $t('employee.unregTitle') }}
                <span v-if="unregCount > 0"
                    class="px-1.5 py-0.5 rounded-full bg-amber-200 text-amber-900 text-[10px] font-bold">{{ unregCount
                    }}</span>
            </button>
        </div>

        <!-- Controls (registered tab only) -->
        <div v-if="activeTab === 'registered'" class="glass rounded-2xl border border-emerald-100 px-4 py-3 space-y-3">
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-3 lg:items-center">
                <div class="lg:col-span-5 relative">
                    <input v-model="search" type="text" :placeholder="$t('employee.searchPlaceholder')"
                        class="w-full h-11 pl-9 pr-8 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm leading-6 text-green-900 placeholder:text-green-900/30">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-emerald-700/40"
                        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                        <path d="M21 21l-4.2-4.2" />
                        <circle cx="11" cy="11" r="6" />
                    </svg>
                    <button v-if="search" @click="search = ''"
                        class="absolute right-2 top-1/2 -translate-y-1/2 w-7 h-7 rounded-full bg-emerald-50 text-emerald-700 flex items-center justify-center text-xs hover:bg-emerald-100">×</button>
                </div>
                <div class="lg:col-span-3">
                    <select v-model="deptFilter"
                        class="w-full h-11 px-3 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">{{ $t('report.allDepartments') }}</option>
                        <option v-for="d in depts" :key="d" :value="d">{{ d }}</option>
                    </select>
                </div>
                <div class="lg:col-span-2">
                    <select v-model="statusFilter"
                        class="w-full h-11 px-3 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">{{ $t('report.allStatuses') }}</option>
                        <option value="0">{{ $t('common.active') }}</option>
                        <option value="1">{{ $t('employee.statusInactive') }}</option>
                    </select>
                </div>
                <div class="lg:col-span-2 flex gap-2">
                    <button @click="clearFilters()"
                        class="flex-1 h-11 px-3 rounded-xl bg-white border-2 border-slate-200 text-slate-700 text-sm font-medium hover:bg-slate-50 inline-flex items-center justify-center">{{
                        $t('common.reset') }}</button>
                    <button @click="listRef?.reload()"
                        class="flex-1 lg:hidden h-11 px-3 rounded-xl bg-emerald-600 text-white text-sm font-bold mono inline-flex items-center justify-center">{{
                        $t('common.load') }}</button>
                </div>
            </div>
            <div v-if="search || deptFilter || statusFilter !== ''"
                class="flex flex-wrap items-center gap-1.5 mono text-[11px]">
                <span class="text-emerald-700/50">{{ $t('common.activeFilters') }}</span>
                <span v-if="search"
                    class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700">Search: "{{
                    search }}"</span>
                <span v-if="deptFilter"
                    class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{ deptFilter
                    }}</span>
                <span v-if="statusFilter !== ''"
                    class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{
                        statusFilter === '0' ? $t('common.active') : $t('employee.statusInactive') }}</span>
                <span class="text-slate-400">- {{ filteredCount }} of {{ total }} rows</span>
            </div>
        </div>

        <!-- Registered tab: list (form is a dialog, mounted once and shown on demand) -->
        <div v-if="activeTab === 'registered'"
            class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden bg-white/80">
            <ListEmployee ref="listRef" :search="search" :dept-filter="deptFilter" :status-filter="statusFilter"
                @edit="cbEdit" @delete="cbDelete" @import="cbImport" />
        </div>

        <!-- Unregistered tab: user ids with punches but no employee row -->
        <div v-if="activeTab === 'unregistered'"
            class="glass rounded-2xl border border-amber-100 shadow-sm overflow-hidden bg-white/80">
            <ListUnregistered ref="unregRef" @link="cbLink" />
        </div>
    </div>

    <FormEmployee ref="compEmployee" @done="() => { listRef?.reload(); unregRef?.reload(); loadUnregCount() }" />
    <SimpleModal :options="sm_data.get()" @ok="handleOk" @cancel="handleCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>
