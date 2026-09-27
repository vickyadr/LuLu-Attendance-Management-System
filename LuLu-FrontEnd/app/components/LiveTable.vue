<script setup>
import { useTransaction } from '~/store/transaction';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
 auth = useAuthStore(),
 config = useRuntimeConfig(),
 list = useTransaction(),
 last_data = ref(0),
 check = useChecker(),
 format = useFormater(),
 contents = computed(()=>{
 return list.contents
 });

async function initLiveTransaction() {
 await $fetch(`${config.public.apiBase}/transaction/live`,
 {
 method: "GET",
 headers: auth.confHeaders(),
 }).then(async (response) => {
 if (response.code == 200) {
 list.set(response.data);
 last_data.value = parseInt(response.message);
 }
 });
};

function parseEnrollType(input){
 return check.inSwitch(input, ["Unknown", "Finger", "PIN", "Card", "Face"])
}
function initials(a,b){
 return `${(a||'')[0]||''}${(b||'')[0]||''}`.toUpperCase().slice(0,2) || '•'
}
function chipCls(t){
 const v = parseEnrollType(t)
 if(v==='Finger') return 'bg-emerald-50 border-emerald-200 text-emerald-700'
 if(v==='Face') return 'bg-lime-50 border-lime-200 text-lime-700'
 if(v==='Card') return 'bg-sky-50 border-sky-200 text-sky-700'
 if(v==='PIN') return 'bg-amber-50 border-amber-200 text-amber-700'
 return 'bg-slate-50 border-slate-200 text-slate-600'
}
function dotCls(t){
 const v = parseEnrollType(t)
 if(v==='Finger') return 'bg-emerald-500'
 if(v==='Face') return 'bg-lime-500'
 if(v==='Card') return 'bg-sky-500'
 if(v==='PIN') return 'bg-amber-500'
 return 'bg-slate-400'
}
function tzShort(tz){
 const n = Number(tz)
 if (Number.isNaN(n)) return ''
 return `GMT${n>=0?'+':''}${n}`
}
function timeParts(ts, tz){
 const n = Number(tz)
 // Device TZ render: enroll_time is a UTC epoch, so shift by the device's own
 // timezone and read back with UTC getters — independent of browser locale.
 if (!Number.isNaN(n)){
   const d = new Date((Number(ts) + n*3600) * 1000)
   const p = (v)=> String(v).padStart(2,'0')
   return {
     d: `${d.getUTCFullYear()}-${p(d.getUTCMonth()+1)}-${p(d.getUTCDate())}`,
     h: `${p(d.getUTCHours())}:${p(d.getUTCMinutes())}:${p(d.getUTCSeconds())}`
   }
 }
 const s = format.stamp_to_naive(ts) // "YYYY-MM-DD HH:MM:SS"
 const [d, h] = s.split(' ')
 return { d: d||'', h: h||'' }
}

onMounted(()=>{
 initLiveTransaction()
});
</script>

<template>
 <div class="rounded-2xl overflow-hidden border border-emerald-100 bg-white shadow-[0_8px_24px_-12px_rgba(16,185,129,.18)] flex flex-col min-h-0 max-h-[62dvh] sm:max-h-[68vh] lg:max-h-none lg:flex-1">
 <!-- header bar -->
 <div class="hidden sm:flex items-center justify-between px-4 py-2.5 bg-gradient-to-r from-emerald-50 to-lime-50/60 border-b border-emerald-100 shrink-0">
 <div class="flex items-center gap-2.5">
 <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse shadow shadow-emerald-200"></span>
 <span class="text-sm font-bold tracking-wide text-green-900 uppercase">Live Transactions</span>
 <span class="hidden sm:inline-flex chip chip-emerald mono !py-1 !text-[9px]">● LIVE FEED</span>
 <span class="mono text-[10px] tracking-widest text-emerald-700/40">• {{ contents.length }} records</span>
 </div>
 <span class="mono text-[10px] tracking-widest text-emerald-700/40 hidden md:inline">{{ contents.length ? 'SYNC OK' : 'IDLE' }}</span>
 </div>

 <div class="min-w-0 min-h-0 scrollbar-thin overflow-x-auto overflow-y-auto overscroll-contain lg:flex-1 lg:min-h-0 lg:overflow-auto">
 <table class="w-full min-w-[560px] border-collapse">
 <thead class="sticky top-0 z-10">
 <tr class="bg-gradient-to-r from-emerald-600 via-emerald-600 to-green-600 text-white">
 <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 w-[22%]">
 <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z"/></svg> Time</span>
 </th>
 <th class="py-2.5 px-4 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 w-[28%]">
 <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M15.75 6a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0zM4.5 20.25a8.25 8.25 0 0116.5 0"/></svg> Employee</span>
 </th>
 <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 w-[18%]">
 <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M20.25 14.15v4.25c0 1.094-.787 2.036-1.872 2.18-2.087.277-4.216.42-6.378.42s-4.291-.143-6.378-.42c-1.085-.144-1.872-1.086-1.872-2.18v-4.25m16.5 0a2.18 2.18 0 00.75-1.661V8.706c0-1.081-.768-2.015-1.837-2.175a48.114 48.114 0 00-3.413-.387m4.5 8.006c-.194.165-.42.295-.673.38A23.978 23.978 0 0112 15.75c-2.648 0-5.195-.429-7.577-1.22a2.016 2.016 0 01-.673-.38m0 0A2.18 2.18 0 013 12.489V8.706c0-1.081.768-2.015 1.837-2.175a48.111 48.111 0 013.413-.387m7.5 0V5.25A2.25 2.25 0 0013.5 3h-3a2.25 2.25 0 00-2.25 2.25v.894m7.5 0a48.667 48.667 0 00-7.5 0"/></svg> Departement</span>
 </th>
 <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 w-[14%]">
 <span class="inline-flex items-center gap-1.5 justify-center w-full"><svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M7.864 4.243A7.5 7.5 0 0119.5 10.5v2.25a7.5 7.5 0 01-7.5 7.5 7.5 7.5 0 01-7.5-7.5v-1.5a7.5 7.5 0 013.636-6.407z"/><path stroke-linecap="round" stroke-linejoin="round" d="M12 12.75a1.5 1.5 0 100-3 1.5 1.5 0 000 3z"/></svg> Method</span>
 </th>
 <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap w-[18%]">
 <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z"/><path stroke-linecap="round" stroke-linejoin="round" d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z"/></svg> Location</span>
 </th>
 </tr>
 </thead>

 <tbody v-if="contents.length > 0">
 <tr v-for="(enroll, index) in contents" :key="enroll.id ?? index" class="group border-b border-emerald-100/60 last:border-0 transition-colors" :class="index % 2 === 0 ? 'bg-white hover:bg-emerald-50/40' : 'bg-emerald-50/20 hover:bg-emerald-50/60'">
 <td class="py-3 px-3 whitespace-nowrap border-r border-emerald-100/60">
 <div class="flex items-center gap-2">
 <span class="mono text-[11px] text-emerald-700/60">{{ timeParts(enroll.date_time, enroll.tz).d }}</span>
 <span class="inline-flex items-center gap-1.5 mono text-xs font-bold text-emerald-700 bg-emerald-50 border border-emerald-200 rounded-full px-2.5 py-1">
 <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 data-tick"></span>
 {{ timeParts(enroll.date_time, enroll.tz).h }}
 </span>
 <span v-if="tzShort(enroll.tz)" class="mono text-[10px] font-bold tracking-widest text-emerald-700/50">{{ tzShort(enroll.tz) }}</span>
 </div>
 </td>
 <td class="py-3 px-4 whitespace-nowrap border-r border-emerald-100/60">
 <div class="flex items-center gap-3">
 <div class="w-8 h-8 rounded-full bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center text-white text-[11px] font-bold shadow-sm shrink-0 ring-1 ring-emerald-200">
 {{ initials(enroll.first_name, enroll.last_name) }}
 </div>
 <div class="leading-tight">
 <p class="text-[13px] font-semibold text-green-900">{{ enroll.first_name }} {{ enroll.last_name }}</p>
 <p class="mono text-[10px] tracking-widest text-emerald-700/50">ID #{{ enroll.enroll_id ?? enroll.id ?? '—' }}</p>
 </div>
 </div>
 </td>
 <td class="py-3 px-3 whitespace-nowrap border-r border-emerald-100/60">
 <span v-if="enroll.departement" class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 mono text-[11px] font-bold">
 <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
 {{ enroll.departement }}
 </span>
 <span v-else class="mono text-[11px] text-slate-400">—</span>
 </td>
 <td class="py-3 px-3 text-center whitespace-nowrap border-r border-emerald-100/60">
 <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border text-[11px] font-bold mono tracking-wide" :class="chipCls(enroll.enroll_type)">
 <span class="w-1.5 h-1.5 rounded-full" :class="dotCls(enroll.enroll_type)"></span>
 {{ parseEnrollType(enroll.enroll_type) }}
 </span>
 </td>
 <td class="py-3 px-3 whitespace-nowrap">
 <span class="inline-flex items-center gap-2 mono text-xs text-emerald-800 bg-white border border-emerald-100 rounded-full px-3 py-1 shadow-sm">
 <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
 {{ enroll.location }}
 </span>
 </td>
 </tr>
 </tbody>

 <tbody v-else>
 <tr>
 <td colspan="5" class="p-0">
 <div class="py-10 px-6 flex flex-col items-center justify-center gap-3 bg-gradient-to-b from-emerald-50/70 via-white to-white tech-grid-soft">
 <div class="hud-frame w-16 h-16 rounded-2xl bg-white border border-emerald-100 flex items-center justify-center text-emerald-500 shadow-sm neon-ring relative overflow-hidden">
 <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>
 <div class="scanline" style="animation-duration:2.8s"></div>
 <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.6"><path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6M12 9v6M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/><path stroke-linecap="round" stroke-linejoin="round" d="M3 8V6a2 2 0 012-2h2M21 8V6a2 2 0 00-2-2h-2M3 16v2a2 2 0 002 2h2M21 16v2a2 2 0 01-2 2h-2"/></svg>
 </div>
 <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">NO LIVE TRANSACTIONS</p>
 <p class="text-sm text-emerald-700/60 text-center max-w-sm">Waiting for the controller to sync — the feed streams in realtime once devices send data.</p>
 <span class="chip chip-emerald mono !text-[9px] mt-1"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 data-tick"></span> STREAMING READY • WS IDLE</span>
 </div>
 </td>
 </tr>
 </tbody>
 </table>
 </div>

 <div v-if="contents.length > 0" class="px-4 py-2.5 bg-emerald-50/50 border-t border-emerald-100 flex items-center justify-between">
 <span class="mono text-[10px] tracking-[0.14em] text-emerald-700/60">● {{ contents.length }} RECORDS • LIVE FEED</span>
 <span class="mono text-[10px] tracking-widest text-emerald-700/40 hidden sm:inline">{{ contents.length }} ROWS</span>
 </div>
 </div>
</template>
