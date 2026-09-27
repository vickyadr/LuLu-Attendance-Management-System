<script setup>
import { useEmployee } from '~/store/employee';
import { useAuthStore } from '~/store/auth';
import { useSchedule } from '~/store/schedule';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useEmployee(),
    sched = useSchedule(),
    emit = defineEmits(['delete','edit']),
    props = defineProps({
        search: { type: String, default: '' },
        deptFilter: { type: String, default: '' },
        statusFilter: { type: String, default: '' },
    });

const scheduleMap = computed(()=>{
    const m={}
    for(const s of (sched.contents||[])){
        if(String(s.parrent)===String(s.id)) m[String(s.parrent)]=s
    }
    return m
})
function scheduleNameFor(id){
    if(id==null||id==='') return '—'
    const s = scheduleMap.value[String(id)]
    return s ? s.name : `#${id}`
}
function scheduleBadgeFor(id){
    const s = scheduleMap.value[String(id)]
    if(!s) return ''
    if(s.type===1) return 'DAILY'
    if(s.type===7) return 'WEEKLY'
    if(s.type===14) return '2M'
    if(s.type===21) return '3M'
    if(s.type===28) return 'MONTHLY'
    return `T${s.type}`
}
function initials(a,b){
    return `${(a||'?')[0]||''}${(b||'')[0]||''}`.toUpperCase().slice(0,2)
}
function avatarBg(item){
    const s=`${item.first_name||''}${item.last_name||''}`
    let h=0; for(let i=0;i<s.length;i++) h=(h*31+s.charCodeAt(i))%360
    return `hsl(${140+(h%40)-20} 70% 38%)`
}

const filtered = computed(()=>{
    let arr = list.contents || []
    const q = (props.search||'').trim().toLowerCase()
    // NOTE: API returns friendly keys via serde rename (id, first_name,
    // last_name, departement, address, status, schedule_id) — NOT employee_*.
    if(q) arr = arr.filter(x => `${x.first_name||''} ${x.last_name||''} ${x.departement||''} ${x.address||''}`.toLowerCase().includes(q))
    if(props.deptFilter) arr = arr.filter(x => (x.departement||'')===props.deptFilter)
    if(props.statusFilter!=='' && props.statusFilter!==null && props.statusFilter!==undefined){
        arr = arr.filter(x => String(x.status)===String(props.statusFilter))
    }
    return arr
})
const total = computed(()=> (list.contents||[]).length)
const filteredCount = computed(()=> filtered.value.length)

function callbackDelete(id){ emit('delete', id) }
function callbackEdit(id){ emit('edit', id) }

async function initEmployee(){
    try{
        const r = await $fetch(`${config.public.apiBase}/employee/list`, { method:'GET', headers: auth.confHeaders() })
        if(r.code===200) list.set(r.data)
    }catch(e){}
    try{
        const r2 = await $fetch(`${config.public.apiBase}/schedule/list`, { method:'GET', headers: auth.confHeaders() })
        if(r2.code===200) sched.set(r2.data)
    }catch(e){}
}
function reload(){ initEmployee() }
defineExpose({ reload, initEmployee })

onMounted(()=>{ initEmployee() })
</script>

<template>
    <div class="flex flex-col overflow-hidden">
        <div class="px-4 py-2.5 bg-gradient-to-r from-emerald-50/70 to-white border-b border-emerald-100 flex items-center justify-between gap-2">
            <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> EMPLOYEE LIST
            </span>
            <span class="mono text-[11px] text-emerald-700/60">{{ filteredCount }} shown • {{ total }} total</span>
        </div>

        <div class="overflow-auto max-h-[52vh] lg:max-h-[56vh] hidden md:block scrollbar-thin">
            <table class="w-full min-w-[760px] border-collapse">
                <thead class="sticky top-0 z-10">
                    <tr class="bg-gradient-to-r from-emerald-600 to-green-600 text-white">
                        <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[30%]">Employee</th>
                        <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[18%]">Department</th>
                        <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[22%]">Schedule</th>
                        <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[12%]">Status</th>
                        <th class="py-2.5 px-3 text-right mono text-[10px] font-bold tracking-[0.14em] uppercase w-[18%]">Actions</th>
                    </tr>
                </thead>
                <tbody v-if="filteredCount>0" class="divide-y divide-emerald-50 bg-white">
                    <tr v-for="(data, idx) in filtered" :key="data.id" class="hover:bg-emerald-50/60 transition" :class="idx%2===0?'bg-white':'bg-emerald-50/20'">
                        <td class="py-3 px-3">
                            <div class="flex items-center gap-3">
                                <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-[11px] font-extrabold shrink-0 shadow-sm ring-1 ring-emerald-200" :style="{background: avatarBg(data)}">{{ initials(data.first_name, data.last_name) }}</div>
                                <div class="min-w-0">
                                    <p class="text-[13px] font-bold text-green-900 leading-none truncate">{{ data.first_name }} {{ data.last_name || '' }}</p>
                                    <p class="mono text-[10px] tracking-wide text-slate-400 mt-1 truncate max-w-[16rem]">#{{ data.id }} • {{ data.address || '— no address' }}</p>
                                </div>
                            </div>
                        </td>
                        <td class="py-3 px-3">
                            <span class="inline-flex px-2.5 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[11px] font-bold">{{ data.departement || '—' }}</span>
                        </td>
                        <td class="py-3 px-3">
                            <span v-if="data.schedule_id" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 mono text-[11px] font-bold">
                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                                {{ scheduleNameFor(data.schedule_id) }}
                                <span class="opacity-60 mono text-[10px]">{{ scheduleBadgeFor(data.schedule_id) }}</span>
                            </span>
                            <span v-else class="inline-flex px-2.5 py-1 rounded-full bg-slate-50 border border-slate-200 text-slate-400 mono text-[11px]">— no schedule</span>
                        </td>
                        <td class="py-3 px-3 text-center">
                            <span class="inline-flex px-2.5 py-1 rounded-full mono text-[11px] font-bold border" :class="String(data.status)==='0' ? 'bg-emerald-500 text-white border-emerald-500' : 'bg-slate-800 text-white border-slate-800'">{{ String(data.status)==='0' ? 'Active' : 'Inactive' }}</span>
                        </td>
                        <td class="py-3 px-3 text-right whitespace-nowrap">
                            <button @click="callbackEdit(data.id)" class="px-3 py-1.5 rounded-full bg-white border-2 border-emerald-100 text-emerald-700 mono text-[11px] font-bold hover:bg-emerald-600 hover:text-white hover:border-emerald-600 transition">Edit</button>
                            <button @click="callbackDelete(data.id)" class="ml-1.5 px-3 py-1.5 rounded-full bg-white border border-red-200 text-red-600 mono text-[11px] font-bold hover:bg-red-600 hover:text-white transition">Delete</button>
                        </td>
                    </tr>
                </tbody>
                <tbody v-else>
                    <tr><td colspan="5" class="py-14 text-center">
                        <div class="flex flex-col items-center gap-3 px-6">
                            <div class="w-12 h-12 rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600">
                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M18 8a2.5 2.5 0 010 5"/><path d="M20 11a4 4 0 01-1 2.5"/></svg>
                            </div>
                            <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">{{ (search||deptFilter||statusFilter!=='') ? 'NOT FOUND' : 'NO EMPLOYEES YET' }}</p>
                            <p class="text-sm text-slate-500 max-w-md">{{ (search||deptFilter||statusFilter!=='') ? 'Try a different keyword, or reset the filters.' : 'Add your first employee in the form on the right — pick a daily/weekly/monthly schedule.' }}</p>
                        </div>
                    </td></tr>
                </tbody>
            </table>
        </div>

        <div class="md:hidden p-3 space-y-3 max-h-[52vh] overflow-auto" v-if="filteredCount>0">
            <div v-for="data in filtered" :key="`m-${data.id}`" class="rounded-2xl border border-emerald-100 bg-white p-3.5 shadow-sm">
                <div class="flex items-start justify-between gap-3">
                    <div class="flex items-center gap-2.5">
                        <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white mono text-xs font-extrabold" :style="{background: avatarBg(data)}">{{ initials(data.first_name, data.last_name) }}</div>
                        <div>
                            <p class="text-sm font-bold text-green-900 leading-none">{{ data.first_name }} {{ data.last_name||'' }}</p>
                            <p class="mono text-[10px] text-slate-400 mt-1">#{{ data.id }}</p>
                        </div>
                    </div>
                    <span class="px-2 py-1 rounded-full mono text-[10px] font-bold border" :class="String(data.status)==='0' ? 'bg-emerald-500 text-white border-emerald-500' : 'bg-slate-800 text-white border-slate-800'">{{ String(data.status)==='0' ? 'Active':'Inactive' }}</span>
                </div>
                <div class="mt-3 flex flex-wrap gap-1.5">
                    <span class="px-2.5 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[11px] font-bold">{{ data.departement||'—' }}</span>
                    <span v-if="data.schedule_id" class="px-2.5 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 mono text-[11px] font-bold">{{ scheduleNameFor(data.schedule_id) }} • {{ scheduleBadgeFor(data.schedule_id) }}</span>
                    <span v-else class="px-2.5 py-1 rounded-full bg-slate-50 border border-slate-200 text-slate-500 mono text-[11px]">— no schedule</span>
                </div>
                <p v-if="data.address" class="mt-2 mono text-xs text-slate-600 line-clamp-2">{{ data.address }}</p>
                <div class="mt-3 flex gap-2">
                    <button @click="callbackEdit(data.id)" class="flex-1 py-2 rounded-xl bg-white border-2 border-emerald-100 text-emerald-700 mono text-xs font-bold hover:bg-emerald-50">Edit</button>
                    <button @click="callbackDelete(data.id)" class="flex-1 py-2 rounded-xl bg-white border border-red-200 text-red-600 mono text-xs font-bold hover:bg-red-50">Delete</button>
                </div>
            </div>
        </div>
        <div v-else class="md:hidden p-10 text-center">
            <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">NO DATA</p>
            <p class="text-sm text-slate-500 mt-1">Adjust the filters, or add an employee.</p>
        </div>

        <div v-if="filteredCount>0" class="px-4 py-2.5 bg-white/90 backdrop-blur border-t border-emerald-100 flex items-center justify-between mono text-[11px]">
            <span class="text-emerald-700/60">{{ filteredCount }} of {{ total }} employees</span>
            <span class="hidden sm:inline text-slate-400">Click Edit to change the schedule • Delete asks for confirmation</span>
        </div>
    </div>
</template>
