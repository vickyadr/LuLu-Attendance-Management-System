<script setup>
import { useAuthStore } from '~/store/auth';
import { useChecker } from '#imports';
import { useShiftStore } from '~/store/shift';
import { faPen, faTrash, faGrip } from '@fortawesome/free-solid-svg-icons';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useShiftStore(),
    check = useChecker(),
    emit = defineEmits(['edit', 'delete']),
    contents = computed(() => list.contents);

function callbackEdit(row) { emit('edit', row) }
function callbackDelete(row) { emit('delete', row) }

async function getListShift() {
    try {
        const response = await $fetch(`${config.public.apiBase}/shift/list`, { method: "GET", headers: auth.confHeaders() });
        if (response.code === 200) list.set(response.data);
    } catch (e) { console.error(e) }
}

function fmt(sec) {
    const s = ((sec % 86400) + 86400) % 86400
    const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
    return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}`
}
function shiftColor(s){
    const h = Math.floor(((s.start_time % 86400)+86400)%86400 / 3600)
    if (h >= 5 && h < 12) return 'bg-amber-400'
    if (h >= 12 && h < 18) return 'bg-emerald-500'
    if (h >= 18 || h < 5) return 'bg-slate-500'
    return 'bg-emerald-500'
}
function handleDragStart(e, row) {
    e.dataTransfer.setData('shift', JSON.stringify({ act:'copy', id: row.id }))
    e.dataTransfer.effectAllowed = 'copy'
    // ghost
    if (e.dataTransfer.setDragImage) {
        const el = document.createElement('div')
        el.textContent = row.name
        el.style.position = 'absolute'; el.style.top='-1000px'
        el.className='px-3 py-1 rounded-full bg-emerald-600 text-white text-xs'
        document.body.appendChild(el)
        e.dataTransfer.setDragImage(el, 0, 0)
        setTimeout(()=> el.remove(), 0)
    }
}

onMounted(()=> getListShift());
defineExpose({ reload: getListShift })
</script>

<template>
    <div>
        <!-- Desktop table -->
        <div class="hidden sm:block overflow-auto max-h-[360px]">
            <table class="w-full">
                <thead class="sticky top-0 z-10">
                    <tr class="bg-gradient-to-r from-emerald-600 to-green-600 text-white">
                        <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-8"></th>
                        <th class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">Shift</th>
                        <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">Work Hours</th>
                        <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 hidden lg:table-cell">Punch Window</th>
                        <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase">Actions</th>
                    </tr>
                </thead>
                <tbody v-if="contents.length>0" class="divide-y divide-emerald-50">
                    <tr v-for="row in contents" :key="row.id"
                        draggable="true" @dragstart="(e)=>handleDragStart(e,row)"
                        class="group hover:bg-emerald-50/50 transition text-xs cursor-grab active:cursor-grabbing">
                        <td class="px-2 py-2.5 text-center">
                            <span class="inline-flex w-7 h-7 rounded-lg bg-white border border-emerald-100 text-emerald-400 group-hover:text-emerald-600 group-hover:border-emerald-200 items-center justify-center">
                                <FontAwesome :icon="faGrip" class="w-3 h-3" />
                            </span>
                        </td>
                        <td class="px-3 py-2.5">
                            <div class="flex items-center gap-2">
                                <span class="w-2 h-7 rounded-full" :class="shiftColor(row)"></span>
                                <div>
                                    <p class="font-bold text-green-900 leading-none">{{ row.name }}</p>
                                    <p class="mono text-[10px] text-emerald-700/50">ID #{{ row.id }}<span v-if="row.nextday" class="ml-1 px-1.5 py-0.5 rounded bg-slate-100 border border-slate-200">+1 day</span><span v-if="row.prevday" class="ml-1 px-1.5 py-0.5 rounded bg-amber-50 border border-amber-200">prev</span></p>
                                </div>
                            </div>
                        </td>
                        <td class="px-3 py-2.5 text-center">
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white border border-emerald-200 mono text-xs font-medium text-green-900">
                                {{ fmt(row.start_time) }} <span class="text-emerald-400">→</span> {{ fmt(row.end_time) }}
                            </span>
                        </td>
                        <td class="px-3 py-2.5 text-center hidden lg:table-cell mono text-[11px] text-emerald-700/70">
                            <div class="flex flex-col gap-0.5 items-center">
                                <span>{{ fmt(row.start_enroll) }} → {{ fmt(row.start_time) }}</span>
                                <span>{{ fmt(row.end_time) }} → {{ fmt(row.end_enroll) }}</span>
                            </div>
                        </td>
                        <td class="px-3 py-2.5 text-center">
                            <div class="inline-flex items-center gap-1">
                                <button @click="callbackEdit(row)" class="w-7 h-7 rounded-lg bg-white border border-emerald-100 text-emerald-600 hover:bg-emerald-600 hover:text-white hover:border-emerald-600 flex items-center justify-center transition" title="Edit"><FontAwesome :icon="faPen" class="w-3 h-3"/></button>
                                <button @click="callbackDelete(row)" class="w-7 h-7 rounded-lg bg-white border border-red-100 text-red-500 hover:bg-red-500 hover:text-white hover:border-red-500 flex items-center justify-center transition" title="Hapus"><FontAwesome :icon="faTrash" class="w-3 h-3"/></button>
                            </div>
                        </td>
                    </tr>
                </tbody>
                <tbody v-else>
                    <tr><td colspan="5" class="py-8 text-center">
                        <p class="mono text-xs text-emerald-700/50">No shifts yet</p>
                        <p class="text-xs text-slate-400 mt-1">Create a shift in the form at the bottom right</p>
                    </td></tr>
                </tbody>
            </table>
        </div>

        <!-- Mobile cards -->
        <div class="sm:hidden p-2 space-y-2 max-h-[360px] overflow-auto">
            <div v-if="contents.length===0" class="py-8 text-center rounded-xl bg-emerald-50/40 border border-emerald-100 mono text-xs text-emerald-700/60">No shifts yet — create one below</div>
            <div v-for="row in contents" :key="row.id"
                 draggable="true" @dragstart="(e)=>handleDragStart(e,row)"
                 class="rounded-2xl border-2 border-emerald-100 bg-white p-3 flex items-center gap-3 active:border-emerald-300 active:bg-emerald-50/40">
                <span class="w-2 h-10 rounded-full shrink-0" :class="shiftColor(row)"></span>
                <div class="flex-1 min-w-0">
                    <p class="font-bold text-sm text-green-900 truncate">{{ row.name }}</p>
                    <p class="mono text-xs text-emerald-700">{{ fmt(row.start_time) }} → {{ fmt(row.end_time) }} <span class="text-emerald-700/40">•</span> {{ fmt(row.start_enroll) }}→{{ fmt(row.end_enroll) }}</p>
                    <p class="mono text-[10px] text-slate-400">ID #{{ row.id }} • drag untuk assign</p>
                </div>
                <div class="flex flex-col gap-1 shrink-0">
                    <button @click="callbackEdit(row)" class="px-3 py-1.5 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold">Edit</button>
                    <button @click="callbackDelete(row)" class="px-3 py-1.5 rounded-full bg-white border border-red-200 text-red-600 mono text-[10px]">Delete</button>
                </div>
            </div>
            <p class="mono text-[10px] text-center text-emerald-700/40 pt-1">Hold & drag a card onto the schedule on the left (desktop), or use the per-day dropdown</p>
        </div>

        <div class="hidden sm:flex px-3 py-2 bg-emerald-50/40 border-t border-emerald-100 mono text-[10px] tracking-wide text-emerald-700/50 items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> Drag <b class="text-emerald-700">⋮⋮</b> onto a day in the schedule on the left • or change it via the per-day dropdown
        </div>
    </div>
</template>
