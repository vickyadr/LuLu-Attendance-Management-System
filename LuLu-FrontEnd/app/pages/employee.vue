<script setup>
import ListEmployee from '~/components/ListEmployee.vue';
import FormEmployee from '~/components/FormEmployee.vue';
import { useEmployee } from '~/store/employee';
import { SMData } from '~/components/SimpleModal.vue';

definePageMeta({ middleware: ["get-auth"], layout: 'default' });
useHead({ title: "LuLu — Employees & Schedules" });

const compEmployee = ref()
const listRef = ref()
const employeeStore = useEmployee()
const sm_data = new SMData()

const search = ref('')
const deptFilter = ref('')
const statusFilter = ref('')

const depts = computed(()=>{
    const s = new Set((employeeStore.contents||[]).map(x=> x.departement).filter(Boolean))
    return [...s].sort()
})
const total = computed(()=> (employeeStore.contents||[]).length)
const aktif = computed(()=> (employeeStore.contents||[]).filter(x=> String(x.status)==='0').length)
const nonaktif = computed(()=> total.value - aktif.value)

function showModal(data){
    sm_data.setText(data.title, data.message)
    sm_data.showOK()
}
function cbEdit(id){
    const all = employeeStore.contents || []
    const found = all.find(x=> String(x.id)===String(id) || String(x.employee_id)===String(id))
    if(found){
        compEmployee.value?.fillForm(found)
        setTimeout(()=>{ document.getElementById('form-employee')?.scrollIntoView({behavior:'smooth', block:'start'}) }, 120)
    } else {
        showModal({title:'Edit', message:'Employee not found'})
    }
}
function cbDelete(id){
    sm_data.setText("Delete Employee", `Employee #${id} will be deleted. Continue?`)
    sm_data.setHelper("del-employee", {id})
    sm_data.showOKCancel()
}
async function handleOk(){
    const helper = sm_data.getHelper("del-employee")
    if(helper) await compEmployee.value?.del_employee(helper.id)
    sm_data.clear()
}
function handleCancel(){ sm_data.clear() }
function clearFilters(){ search.value=''; deptFilter.value=''; statusFilter.value='' }

const filteredCount = computed(()=>{
    let arr = employeeStore.contents||[]
    const q=(search.value||'').trim().toLowerCase()
    if(q) arr=arr.filter(x=> `${x.first_name||''} ${x.last_name||''} ${x.departement||''}`.toLowerCase().includes(q))
    if(deptFilter.value) arr=arr.filter(x=> (x.departement||'')===deptFilter.value)
    if(statusFilter.value!=='') arr=arr.filter(x=> String(x.status)===String(statusFilter.value))
    return arr.length
})
</script>

<template>
    <div class="space-y-4">
        <!-- Header -->
        <div class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col lg:flex-row lg:items-center gap-4 tech-grid-soft">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3">
                <div class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="1.8"><path d="M16 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 11a3 3 0 00-3-3 3 3 0 00-3 3 3 3 0 003 3 3 3 0 003-3z"/><path d="M15 11h3"/><path d="M16.5 9.5v3"/></svg>
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">Employees</h1>
                    <p class="text-xs text-emerald-700/60 mt-0.5">Manage employees & schedule assignment — pick a daily/weekly/monthly roster, no printing or exporting needed, it feeds the reports directly.</p>
                    <div class="mt-2 flex flex-wrap items-center gap-2 mono text-[10px]">
                        <span class="chip chip-emerald">{{ total }} employees</span>
                        <span class="inline-flex items-center gap-1.5 text-emerald-700/40"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> {{ aktif }} active • {{ nonaktif }} inactive</span>
                        <span class="hidden sm:inline-flex chip bg-white border-emerald-200 text-emerald-700 border">SCHEDULE • SHIFT</span>
                    </div>
                </div>
            </div>
            <div class="lg:ml-auto flex items-center gap-2">
                <div class="hidden sm:flex items-center gap-2 px-3 py-2 rounded-xl bg-white border border-emerald-100 mono text-[10px]">
                    <span class="w-2 h-2 rounded-full bg-emerald-500"></span> {{ total }} REGISTERED
                    <span class="w-px h-3 bg-emerald-100"></span>
                    <span class="text-emerald-700">{{ aktif }} ACTIVE</span>
                </div>
                <button @click="listRef?.reload()" class="hidden sm:inline-flex px-3 py-2 rounded-xl bg-emerald-600 text-white mono text-xs font-bold hover:bg-emerald-700">Reload</button>
            </div>
        </div>

        <!-- Controls -->
        <div class="glass rounded-2xl border border-emerald-100 px-4 py-3 space-y-3">
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-3 lg:items-center">
                <div class="lg:col-span-5 relative">
                    <input v-model="search" type="text" placeholder="Search name / department / address…" class="w-full h-11 pl-9 pr-8 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm leading-6 text-green-900 placeholder:text-green-900/30">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-emerald-700/40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 21l-4.2-4.2"/><circle cx="11" cy="11" r="6"/></svg>
                    <button v-if="search" @click="search=''" class="absolute right-2 top-1/2 -translate-y-1/2 w-7 h-7 rounded-full bg-emerald-50 text-emerald-700 flex items-center justify-center text-xs hover:bg-emerald-100">×</button>
                </div>
                <div class="lg:col-span-3">
                    <select v-model="deptFilter" class="w-full h-11 px-3 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">All departments</option>
                        <option v-for="d in depts" :key="d" :value="d">{{ d }}</option>
                    </select>
                </div>
                <div class="lg:col-span-2">
                    <select v-model="statusFilter" class="w-full h-11 px-3 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">All statuses</option>
                        <option value="0">Active</option>
                        <option value="1">Inactive</option>
                    </select>
                </div>
                <div class="lg:col-span-2 flex gap-2">
                    <button @click="clearFilters()" class="flex-1 h-11 px-3 rounded-xl bg-white border-2 border-slate-200 text-slate-700 text-sm font-medium hover:bg-slate-50 inline-flex items-center justify-center">Reset</button>
                    <button @click="listRef?.reload()" class="flex-1 lg:hidden h-11 px-3 rounded-xl bg-emerald-600 text-white text-sm font-bold mono inline-flex items-center justify-center">Load</button>
                </div>
            </div>
            <div v-if="search || deptFilter || statusFilter!==''" class="flex flex-wrap items-center gap-1.5 mono text-[11px]">
                <span class="text-emerald-700/50">Active filters:</span>
                <span v-if="search" class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700">Search: "{{ search }}"</span>
                <span v-if="deptFilter" class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{ deptFilter }}</span>
                <span v-if="statusFilter!==''" class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{ statusFilter==='0'?'Active':'Inactive' }}</span>
                <span class="text-slate-400">— {{ filteredCount }} of {{ total }} rows</span>
            </div>
        </div>

        <!-- Main grid -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
            <!-- List — 7 cols -->
            <div class="lg:col-span-7 xl:col-span-7">
                <div class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden bg-white/80">
                    <ListEmployee ref="listRef" :search="search" :dept-filter="deptFilter" :status-filter="statusFilter" @edit="cbEdit" @delete="cbDelete" />
                </div>
                <p class="mt-2 mono text-[10px] tracking-wide text-emerald-700/40 px-1">Tip: filters help once you have many employees • click Edit to change a schedule without deleting it • Inactive status still keeps the history</p>
            </div>
            <!-- Form — 5 cols -->
            <div id="form-employee" class="lg:col-span-5 xl:col-span-5 space-y-4">
                <div class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden bg-white">
                    <FormEmployee ref="compEmployee" @notif="showModal" @done="()=>{}" />
                </div>
                <div class="rounded-2xl border border-emerald-100 bg-emerald-50/40 px-4 py-3.5">
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> SCHEDULE ASSIGNMENT</p>
                    <ol class="mt-2 space-y-1.5 mono text-[11px] leading-relaxed text-emerald-800/80 list-decimal list-inside">
                        <li>Create <b>Shift</b> first (work hours and punch windows).</li>
                        <li>Create a <b>schedule</b> in the Shift menu → pick a Daily/Weekly/Monthly pattern with day off.</li>
                        <li>Here you pick the schedule for each employee — the roster preview appears automatically.</li>
                        <li>Change a schedule any time via Edit — reports still respect the history per date.</li>
                    </ol>
                    <div class="mt-3 flex flex-wrap gap-1.5">
                        <span class="chip chip-emerald mono !text-[9px]">DAILY</span>
                        <span class="chip bg-white border-emerald-200 text-emerald-700 border mono !text-[9px]">WEEKLY</span>
                        <span class="chip bg-white border-emerald-200 text-emerald-700 border mono !text-[9px]">MONTHLY</span>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <SimpleModal :options="sm_data.get()" @ok="handleOk" @cancel="handleCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>
