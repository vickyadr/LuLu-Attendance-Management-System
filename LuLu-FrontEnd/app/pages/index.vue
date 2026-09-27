<script setup>
import LiveTable from '~/components/LiveTable.vue';
import { useAuthStore } from '~/store/auth';
import { useChecker } from '#imports';

useHead({ title: "LuLu — Dashboard" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

const auth = useAuthStore()
const config = useRuntimeConfig()
const check = useChecker()

const greeting = computed(()=>{
    const h = new Date().getHours()
    if(h<11) return 'Good morning'
    if(h<15) return 'Good afternoon'
    if(h<18) return 'Good evening'
    return 'Good night'
})
const todayStr = computed(()=>{
    try{ return new Intl.DateTimeFormat('en-GB', { weekday:'long', day:'numeric', month:'long', year:'numeric' }).format(new Date()) }catch{ return '' }
})
// Store exposes displayName/initials as plain unwrapped computeds; reading
// auth.user.fname.value here returned undefined and silently showed the fallback.
const displayName = computed(()=> auth.displayName)

const stats = reactive({
    karyawan: { total: 0, aktif: 0, loading: true },
    device: { total: 0, online: 0, loading: true },
    today: { total: 0, hadir: 0, terlambat: 0, puncht: 0, early: 0, overtime: 0, loading: true },
    live: { count: 0, loading: true },
})
const lastSync = ref('—')
const refreshing = ref(false)

function statusTagsOf(it){
    const tags=[]
    const dur = (it.shift_end ?? 0) - (it.shift_start ?? 0)
    const isAbsen = check.isNull(it.start) && check.isNull(it.end)
    if(it.late_time>0) tags.push('late')
    if(!isAbsen && (it.working_time||0) < dur) tags.push('early')
    if(isAbsen) tags.push('puncht')
    if((it.working_time||0) > dur) tags.push('overtime')
    if(tags.length===0 && !isAbsen) tags.push('present')
    return tags
}

async function fetchKaryawan(){
    stats.karyawan.loading=true
    try{
        const r = await $fetch(`${config.public.apiBase}/employee/list`, { method:'GET', headers: auth.confHeaders() })
        if(r.code===200){
            const arr = r.data||[]
            stats.karyawan.total = arr.length
            stats.karyawan.aktif = arr.filter(x=> String(x.status)==='0').length
        }
    }catch(e){} finally{ stats.karyawan.loading=false }
}
async function fetchDevices(){
    stats.device.loading=true
    try{
        const r = await $fetch(`${config.public.apiBase}/device/list`, { method:'GET', headers: auth.confHeaders() })
        if(r.code===200){
            const arr = r.data||[]
            stats.device.total = arr.length
            stats.device.online = arr.filter(d=> d.status===1 || String(d.status)==='1').length
        }
    }catch(e){} finally{ stats.device.loading=false }
}
async function fetchToday(){
    stats.today.loading=true
    try{
        const r = await $fetch(`${config.public.apiBase}/report/today`, { method:'GET', headers: auth.confHeaders() })
        if(r.code===200){
            const arr = r.data||[]
            stats.today.total = arr.length
            let hadir=0, terlambat=0, puncht=0, early=0, overtime=0
            for(const it of arr){
                const tags = statusTagsOf(it)
                if(tags.includes('present')) hadir++
                if(tags.includes('late')) terlambat++
                if(tags.includes('puncht')) puncht++
                if(tags.includes('early')) early++
                if(tags.includes('overtime')) overtime++
            }
            stats.today.hadir = hadir
            stats.today.terlambat = terlambat
            stats.today.puncht = puncht
            stats.today.early = early
            stats.today.overtime = overtime
        }
    }catch(e){} finally{ stats.today.loading=false }
}
async function fetchLive(){
    stats.live.loading=true
    try{
        const r = await $fetch(`${config.public.apiBase}/transaction/live`, { method:'GET', headers: auth.confHeaders() })
        if(r.code===200){
            stats.live.count = (r.data||[]).length
            lastSync.value = new Date().toLocaleTimeString('en-GB', { hour:'2-digit', minute:'2-digit' })
        }
    }catch(e){} finally{ stats.live.loading=false }
}

async function refreshAll(){
    refreshing.value=true
    await Promise.allSettled([fetchKaryawan(), fetchDevices(), fetchToday(), fetchLive()])
    refreshing.value=false
}

onMounted(()=>{ refreshAll() })
</script>

<template>
    <div class="flex flex-col gap-3 lg:flex-1 lg:min-h-0">
        <!-- Greeting / Hero — compact so viewport fits without scrolling on 900px -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-3 shrink-0">
            <!-- Welcome — 5 cols -->
            <div class="lg:col-span-5 hud-frame rounded-2xl p-3 sm:p-4 bg-gradient-to-br from-emerald-600 via-emerald-600 to-green-700 text-white relative overflow-hidden card-lift neon-ring">
                <div class="pointer-events-none absolute inset-0 tech-dots opacity-10"></div>
                <div class="scanline" style="animation-duration:4s"></div>
                <div class="absolute -right-10 -top-10 w-44 h-44 bg-white/10 rounded-full blur-2xl"></div>
                <div class="absolute -left-8 -bottom-8 w-36 h-36 bg-lime-300/20 rounded-full blur-xl"></div>
                <div class="relative">
                    <div class="flex items-center gap-2 flex-wrap">
                        
                        <span class="mono text-[10px] tracking-[0.14em] text-white/60 hidden sm:inline">BIOMETRIC</span>
                        <span class="ml-auto mono text-[10px] tracking-widest text-white/60 hidden sm:inline-flex items-center gap-1.5"><span class="w-1.5 h-1.5 rounded-full bg-lime-300 animate-pulse"></span> {{ lastSync }} • SYNC</span>
                    </div>
                    <p class="mono text-white/70 text-[11px] tracking-[0.14em] uppercase mt-3">{{ todayStr }}</p>
                    <h2 class="display text-[22px] sm:text-2xl font-extrabold tracking-tight mt-1 leading-none">{{ greeting }},<br><span class="text-white">{{ displayName }}</span> <span class="mono text-[11px] font-bold tracking-widest text-white/60 align-middle ml-1">◉ LIVE</span></h2>
                    <p class="text-emerald-100/80 text-[13px] mt-2 leading-relaxed">Manage attendance — daily/weekly/monthly rosters, realtime punch data, export-ready reports.</p>
                    <div class="mt-4 flex flex-wrap gap-2">
                        <button @click="navigateTo('/report')" class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white text-emerald-700 mono text-xs font-extrabold hover:bg-emerald-50 transition shadow-sm">Open Report <span class="w-5 h-5 rounded-full bg-emerald-600 text-white flex items-center justify-center text-[10px]">→</span></button>
                        <button @click="navigateTo('/employee')" class="inline-flex items-center gap-1.5 px-4 py-2 rounded-full bg-white/15 backdrop-blur text-white mono text-xs font-bold border border-white/20 hover:bg-white/20 transition">+ Employee</button>
                    </div>
                    <div class="mt-4 flex flex-wrap gap-1.5 mono text-[10px]">
                        <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/10 border border-white/20 text-white/80"><span class="w-1.5 h-1.5 rounded-full bg-lime-300"></span> STREAMING</span>
                        <span class="inline-flex px-2.5 py-1 rounded-full bg-white/10 text-white/70">REALTIME FEED</span>
                        <button @click="refreshAll()" :disabled="refreshing" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/10 hover:bg-white/15 text-white mono text-[10px] font-bold disabled:opacity-60">
                            <svg v-if="refreshing" class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"/><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"/></svg>
                            <span v-else class="w-3 h-3 flex items-center justify-center">↻</span> Refresh
                        </button>
                    </div>
                </div>
            </div>

            <!-- Stats — 7 cols : 2x2 -->
            <div class="lg:col-span-7 grid grid-cols-2 gap-2 sm:gap-3 content-start">
                <!-- Employees -->
                <button @click="navigateTo('/employee')" class="text-left hud-frame glass rounded-2xl p-3 sm:p-4 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div class="w-10 h-10 rounded-xl bg-emerald-100 flex items-center justify-center text-emerald-600 group-hover:bg-emerald-600 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z"/></svg>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-3 flex items-center gap-1.5">EMPLOYEES <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span></p>
                    <p v-if="stats.karyawan.loading" class="mt-1 h-7 w-12 rounded bg-emerald-100 animate-pulse"></p>
                    <p v-else class="text-2xl font-extrabold text-green-900 mt-1 tracking-tight">{{ stats.karyawan.total }}</p>
                    <p v-if="!stats.karyawan.loading" class="mono text-[11px] text-emerald-700/60 mt-1">{{ stats.karyawan.aktif }} active • {{ Math.max(0, stats.karyawan.total - stats.karyawan.aktif) }} inactive</p>
                    <p v-else class="mt-1 h-3 w-20 rounded bg-emerald-50 animate-pulse"></p>
                    <span class="absolute top-3 right-3 w-6 h-6 rounded-full bg-emerald-50 border border-emerald-100 text-emerald-600 flex items-center justify-center mono text-[10px] group-hover:bg-emerald-600 group-hover:text-white transition">↗</span>
                </button>

                <!-- Device -->
                <button @click="navigateTo('/controller')" class="text-left hud-frame glass rounded-2xl p-3 sm:p-4 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div class="w-10 h-10 rounded-xl bg-sky-50 border border-sky-100 flex items-center justify-center text-sky-600 group-hover:bg-sky-600 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="14" rx="2"/><path d="M8 20h8"/><path d="M12 16v4"/></svg>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-3">DEVICES</p>
                    <p v-if="stats.device.loading" class="mt-1 h-7 w-12 rounded bg-sky-100 animate-pulse"></p>
                    <p v-else class="text-2xl font-extrabold text-green-900 mt-1 tracking-tight">{{ stats.device.total }}</p>
                    <p v-if="!stats.device.loading" class="mono text-[11px] mt-1" :class="stats.device.online>0 ? 'text-emerald-600' : 'text-amber-600'"><span class="inline-flex items-center gap-1"><span class="w-1.5 h-1.5 rounded-full" :class="stats.device.online>0?'bg-emerald-500':'bg-amber-500'"></span> {{ stats.device.online }} online • {{ Math.max(0, stats.device.total - stats.device.online) }} offline</span></p>
                    <p v-else class="mt-1 h-3 w-20 rounded bg-sky-50 animate-pulse"></p>
                    <span class="absolute top-3 right-3 w-6 h-6 rounded-full bg-sky-50 border border-sky-100 text-sky-600 flex items-center justify-center mono text-[10px] group-hover:bg-sky-600 group-hover:text-white transition">↗</span>
                </button>

                <!-- Today -->
                <button @click="navigateTo('/report')" class="text-left hud-frame glass rounded-2xl p-3 sm:p-4 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div class="w-10 h-10 rounded-xl bg-amber-50 border border-amber-100 flex items-center justify-center text-amber-600 group-hover:bg-amber-500 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="16" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/><path d="M9 16l2 2 4-4"/></svg>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-3">TODAY</p>
                    <p v-if="stats.today.loading" class="mt-1 h-7 w-16 rounded bg-amber-100 animate-pulse"></p>
                    <p v-else class="text-2xl font-extrabold text-green-900 mt-1 tracking-tight">{{ stats.today.total }} <span class="mono text-xs font-bold text-emerald-700/40">records</span></p>
                    <p v-if="!stats.today.loading" class="mono text-[11px] text-slate-600 mt-1 flex flex-wrap gap-1">
                        <span class="inline-flex px-1.5 py-0.5 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 font-bold">{{ stats.today.hadir }} present</span>
                        <span v-if="stats.today.terlambat>0" class="inline-flex px-1.5 py-0.5 rounded-full bg-red-50 border border-red-200 text-red-600 font-bold">{{ stats.today.terlambat }} telat</span>
                        <span v-else class="inline-flex px-1.5 py-0.5 rounded-full bg-white border border-emerald-100 text-emerald-700/60">0 late</span>
                    </p>
                    <p v-else class="mt-1 h-3 w-24 rounded bg-amber-50 animate-pulse"></p>
                    <span class="absolute top-3 right-3 w-6 h-6 rounded-full bg-amber-50 border border-amber-100 text-amber-600 flex items-center justify-center mono text-[10px] group-hover:bg-amber-500 group-hover:text-white transition">↗</span>
                </button>

                <!-- Live -->
                <div class="hud-frame glass rounded-2xl p-4 sm:p-5 border border-emerald-100 relative overflow-hidden flex flex-col">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div class="w-10 h-10 rounded-xl bg-lime-100 border border-lime-200 flex items-center justify-center text-lime-700 relative">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><path d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z"/></svg>
                        <span class="absolute w-2 h-2 rounded-full bg-lime-500 animate-pulse -translate-y-1 translate-x-3"></span>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-3 flex items-center gap-1.5"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> LIVE FEED</p>
                    <p v-if="stats.live.loading" class="mt-1 h-7 w-12 rounded bg-lime-100 animate-pulse"></p>
                    <p v-else class="text-2xl font-extrabold text-green-900 mt-1 tracking-tight">{{ stats.live.count }} <span class="mono text-xs font-bold text-emerald-700/40">transactions</span></p>
                    <p class="mono text-[11px] text-emerald-700/50 mt-1">Latest 100 • synced {{ lastSync }}</p>
                </div>
            </div>
        </div>

        <!-- Quick actions -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 shrink-0">
            <button @click="navigateTo('/employee')" class="group flex items-center gap-3 px-3 py-2.5 rounded-2xl bg-white border-2 border-emerald-100 hover:border-emerald-300 hover:bg-emerald-50/60 transition text-left">
                <span class="w-9 h-9 rounded-xl bg-emerald-600 text-white flex items-center justify-center shrink-0">＋</span>
                <span>
                    <span class="block text-sm font-bold text-green-900">Employees</span>
                    <span class="block mono text-[11px] text-emerald-700/50">Add & assign schedules</span>
                </span>
                <span class="ml-auto mono text-emerald-700/30 group-hover:text-emerald-600">→</span>
            </button>
            <button @click="navigateTo('/shift')" class="group flex items-center gap-3 px-3 py-2.5 rounded-2xl bg-white border-2 border-emerald-100 hover:border-emerald-300 hover:bg-emerald-50/60 transition text-left">
                <span class="w-9 h-9 rounded-xl bg-white border border-emerald-200 text-emerald-700 flex items-center justify-center shrink-0">◷</span>
                <span>
                    <span class="block text-sm font-bold text-green-900">Shifts & Schedules</span>
                    <span class="block mono text-[11px] text-emerald-700/50">Daily • Weekly • Monthly</span>
                </span>
                <span class="ml-auto mono text-emerald-700/30 group-hover:text-emerald-600">→</span>
            </button>
            <button @click="navigateTo('/controller')" class="group flex items-center gap-3 px-3 py-2.5 rounded-2xl bg-white border-2 border-emerald-100 hover:border-sky-200 hover:bg-sky-50/40 transition text-left">
                <span class="w-9 h-9 rounded-xl bg-sky-600 text-white flex items-center justify-center shrink-0">⬡</span>
                <span>
                    <span class="block text-sm font-bold text-green-900">Controller</span>
                    <span class="block mono text-[11px] text-emerald-700/50">{{ stats.device.total }} device • {{ stats.device.online }} online</span>
                </span>
                <span class="ml-auto mono text-emerald-700/30 group-hover:text-sky-600">→</span>
            </button>
            <button @click="navigateTo('/report')" class="group flex items-center gap-3 px-3 py-2.5 rounded-2xl bg-white border-2 border-emerald-100 hover:border-amber-200 hover:bg-amber-50/40 transition text-left">
                <span class="w-9 h-9 rounded-xl bg-amber-500 text-white flex items-center justify-center shrink-0">▤</span>
                <span>
                    <span class="block text-sm font-bold text-green-900">Reports</span>
                    <span class="block mono text-[11px] text-emerald-700/50">Today • 7/30 days • Export</span>
                </span>
                <span class="ml-auto mono text-emerald-700/30 group-hover:text-amber-600">→</span>
            </button>
        </div>

        <!-- Live Transactions -->
        <LiveTable/>

        <p class="mono text-[10px] tracking-wide text-emerald-700/40 px-1 text-center sm:text-left">Tip: today's numbers come from shift hours — if empty, check the enrolment window on the device, or change the range in Reports to see history • Auto-refreshes when the dashboard opens</p>
    </div>
</template>
