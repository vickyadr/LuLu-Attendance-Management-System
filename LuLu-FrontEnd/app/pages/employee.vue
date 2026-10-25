<script setup>
import ListEmployee from '~/components/ListEmployee.vue';
import FormEmployee from '~/components/FormEmployee.vue';
import ListUnregistered from '~/components/ListUnregistered.vue';
import { useEmployee } from '~/store/employee';
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';
import { SMData } from '~/components/SimpleModal.vue';
import { faUserPlus, faMagnifyingGlass, faXmark, faUsers, faUserClock, faRotate } from '@fortawesome/free-solid-svg-icons';

definePageMeta({ middleware: ["get-auth"], layout: 'default' });
useHead({ title: "LuLu - Employees & Schedules" });

const { t } = useI18n()
const compEmployee = ref()
const listRef = ref()
const unregRef = ref()
const employeeStore = useEmployee()
const notify = useNotifyStore()
const sm_data = new SMData()
// The list tells us when its request failed, so the header cannot keep showing
// "0 employees" as if the database were empty.
const listError = ref(false)

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

// Counts and department options come from the list endpoint, which counts the
// whole table; the page itself only holds 50 rows now.
const depts = computed(() => employeeStore.meta.departments || [])
const total = computed(() => employeeStore.meta.all_total || 0)
const aktif = computed(() => employeeStore.meta.active || 0)
const nonaktif = computed(() => employeeStore.meta.inactive || 0)

// Import reports per-row failures as a list, not a single message: the operator
// needs the row number to fix the file, and a toast that truncates would hide
// which lines to look at.
function cbImport(res) {
    const d = res?.data?.[0] || res?.data || {}
    const errs = d.errors || []
    if (res?.code === 200 && errs.length === 0) {
        notify.success(t('employee.importTitle'), t('employee.imported', { n: d.inserted || 0 }))
    } else if (res?.code === 200) {
        const head = errs.slice(0, 5).map(e => `${t('employee.importRow', { n: e.row })}: ${e.field} ${e.reason}`).join('\n')
        const more = errs.length > 5 ? `\n+${errs.length - 5} more` : ''
        notify.warning(t('employee.importTitle'), t('employee.importedFailed', { n: d.inserted || 0, m: errs.length }) + `\n${head}${more}`, 0)
    } else {
        notify.error(t('employee.importTitle'), res?.message || t('common.importFailed'))
    }
}

function cbEdit(id) {
    const all = employeeStore.contents || []
    const found = all.find(x => String(x.id) === String(id) || String(x.employee_id) === String(id))
    if (found) { compEmployee.value?.fillForm(found); compEmployee.value?.open() }
    else showModal({ title: $t('common.edit'), message: $t('employee.notFound') })
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
    const emp = (employeeStore.contents || []).find(x => String(x.id) === String(id))
    const name = emp ? `${emp.first_name || ''} ${emp.last_name || ''}`.trim() : ''
    sm_data.setText(t('employee.deleteTitle'), t('employee.deleteBody', { name: name || `#${id}` }))
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

const filteredCount = computed(() => employeeStore.meta.total || 0)
onMounted(() => loadUnregCount())
</script>

<template>
    <div class="flex flex-col gap-4">

        <!-- page head -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
            <div class="min-w-0">
                <h1 class="display text-[20px] text-text">{{ $t('employee.title') }}</h1>
                <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                    <span v-if="!listError" class="chip chip-quiet">{{ total }} {{ $t('employee.employeesUnit')
                        }}</span>
                    <span v-if="!listError" class="chip chip-ok">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
                        {{ $t('employee.statActiveInactive', { active: aktif, inactive: nonaktif }) }}
                    </span>
                    <span v-if="!listError" class="chip chip-quiet hidden sm:inline-flex">{{
                        $t('employee.statRegistered', { total: total }) }}</span>
                </div>
            </div>
            <div class="sm:ml-auto">
                <button v-if="canAdd" @click="cbNew()" class="btn btn-primary w-full sm:w-auto">
                    <font-awesome :icon="faUserPlus" class="w-3.5 h-3.5" />
                    {{ $t('employee.addTitle') }}
                </button>
            </div>
        </div>

        <!-- tabs: registered employees vs unregistered device user ids -->
        <div
            class="flex flex-wrap items-center gap-1.5 p-1 rounded-2xl border border-border bg-surface w-fit max-w-full">
            <button @click="activeTab = 'registered'" class="btn !px-3 sm:!px-4"
                :class="activeTab === 'registered' ? 'btn-primary' : 'btn-quiet'">
                <font-awesome :icon="faUsers" class="w-3.5 h-3.5" />
                {{ $t('employee.listTitle') }}
                <span class="num text-[11px] leading-none px-1.5 py-0.5 rounded-md"
                    :class="activeTab === 'registered' ? 'bg-white/20' : 'bg-surface-raised'">{{ total }}</span>
            </button>
            <button @click="activeTab = 'unregistered'" class="btn !px-3 sm:!px-4"
                :class="activeTab === 'unregistered' ? 'btn-primary' : 'btn-quiet'">
                <font-awesome :icon="faUserClock" class="w-3.5 h-3.5" />
                {{ $t('employee.unregTitle') }}
                <span v-if="unregCount > 0" class="chip chip-warn">{{ unregCount }}</span>
            </button>
        </div>

        <!-- controls, registered tab only -->
        <div v-if="activeTab === 'registered'" class="panel p-3 sm:p-4 space-y-3">
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-2.5 lg:items-center">
                <div class="lg:col-span-5 relative">
                    <font-awesome :icon="faMagnifyingGlass"
                        class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
                    <input v-model="search" type="text" :placeholder="$t('employee.searchPlaceholder')"
                        class="field has-icon-left has-icon-right" :aria-label="$t('employee.searchPlaceholder')">
                    <button v-if="search" @click="search = ''" :aria-label="$t('common.clearSearch')"
                        class="absolute right-1.5 top-1/2 -translate-y-1/2 w-9 h-9 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised transition">
                        <font-awesome :icon="faXmark" class="w-3 h-3" />
                    </button>
                </div>
                <div class="lg:col-span-3">
                    <select v-model="deptFilter" class="field" :aria-label="$t('report.allDepartments')">
                        <option value="">{{ $t('report.allDepartments') }}</option>
                        <option v-for="d in depts" :key="d" :value="d">{{ d }}</option>
                    </select>
                </div>
                <div class="lg:col-span-2">
                    <select v-model="statusFilter" class="field" :aria-label="$t('report.allStatuses')">
                        <option value="">{{ $t('report.allStatuses') }}</option>
                        <option value="0">{{ $t('common.active') }}</option>
                        <option value="1">{{ $t('employee.statusInactive') }}</option>
                    </select>
                </div>
                <div class="lg:col-span-2 flex flex-col sm:flex-row gap-2">
                    <button @click="clearFilters()" class="btn btn-secondary flex-1">{{ $t('common.reset') }}</button>
                    <button @click="listRef?.reload()" class="btn btn-primary flex-1 lg:hidden">
                        <font-awesome :icon="faRotate" class="w-3.5 h-3.5" />
                        {{ $t('common.load') }}
                    </button>
                </div>
            </div>

            <div v-if="search || deptFilter || statusFilter !== ''" class="flex flex-wrap items-center gap-1.5">
                <span class="mono text-[10px] font-bold tracking-[0.12em] uppercase text-text-muted">{{
                    $t('common.activeFilters') }}</span>
                <span v-if="search" class="chip chip-quiet">"{{ search }}"</span>
                <span v-if="deptFilter" class="chip chip-quiet">{{ deptFilter }}</span>
                <span v-if="statusFilter !== ''" class="chip chip-quiet">{{ statusFilter === '0' ? $t('common.active') :
                    $t('employee.statusInactive') }}</span>
                <span class="mono text-[10.5px] text-text-muted">{{ filteredCount }} / {{ total }}</span>
            </div>
        </div>

        <!-- registered tab: list. the form is a dialog, mounted once and shown on demand -->
        <div v-if="activeTab === 'registered'" class="panel overflow-hidden">
            <ListEmployee ref="listRef" :search="search" :dept-filter="deptFilter" :status-filter="statusFilter"
                @edit="cbEdit" @delete="cbDelete" @import="cbImport" @load-error="listError = $event" />
        </div>

        <!-- unregistered tab: user ids with punches but no employee row -->
        <div v-if="activeTab === 'unregistered'" class="panel overflow-hidden">
            <ListUnregistered ref="unregRef" @link="cbLink" />
        </div>
    </div>

    <FormEmployee ref="compEmployee" @done="() => { listRef?.reload(); unregRef?.reload(); loadUnregCount() }" />
    <SimpleModal :options="sm_data.get()" @ok="handleOk" @cancel="handleCancel" v-show="sm_data.isShow()" />
</template>
