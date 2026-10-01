<script setup>
import { useDeviceStore } from '~/store/devices';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';
import { faMagnifyingGlass, faPen, faTrash, faRotate, faServer } from '@fortawesome/free-solid-svg-icons';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useDeviceStore(),
    check = useChecker(),
    format = useFormater(),
    emit = defineEmits(['delete', 'edit']),
    search = ref(''),
    statusFilter = ref('all'),
    loading = ref(false);

const contents = computed(() => list.contents)
const filtered = computed(() => {
    let out = contents.value
    const q = search.value.trim().toLowerCase()
    if (q) out = out.filter(d =>
        (d.name || '').toLowerCase().includes(q) ||
        (d.sn || '').toLowerCase().includes(q) ||
        (d.location || '').toLowerCase().includes(q)
    )
    if (statusFilter.value !== 'all') {
        const map = { online: 1, offline: 2, disable: 3 }
        const code = map[statusFilter.value]
        if (code !== undefined) out = out.filter(d => Number(d.status) === code)
    }
    return out
})
const hasFilter = computed(() => search.value.trim() !== '' || statusFilter.value !== 'all')
function clearFilter() { search.value = ''; statusFilter.value = 'all' }

async function initListController() {
    loading.value = true
    try {
        const response = await $fetch(`${config.public.apiBase}/device/list`, { method: "GET", headers: auth.confHeaders() });
        if (response.code == 200) list.set(response.data);
    } catch (e) { console.error(e) } finally { loading.value = false }
}

function callbackDelete(id) { emit('delete', id) }
function callbackEdit(id) { emit('edit', id) }

function statusMeta(s) {
    const n = Number(s)
    switch (n) {
        case 1: return { label: 'Online', cls: 'bg-emerald-500 text-white border-emerald-500', dot: 'bg-white', ring: 'ring-emerald-200' }
        case 2: return { label: 'Offline', cls: 'bg-amber-500 text-white border-amber-500', dot: 'bg-white', ring: 'ring-amber-200' }
        case 3: return { label: 'Disable', cls: 'bg-slate-400 text-white border-slate-400', dot: 'bg-white', ring: 'ring-slate-200' }
        case 4: return { label: 'Sync', cls: 'bg-sky-500 text-white border-sky-500', dot: 'bg-white', ring: 'ring-sky-200' }
        default: return { label: 'Unknown', cls: 'bg-white text-slate-500 border-slate-200', dot: 'bg-slate-300', ring: '' }
    }
}
function handlerLabel(h) {
    if (h == null || h === '') return '—'
    const s = String(h).toLowerCase()
    if (s === '4' || s.includes('iclock') || s.includes('adms')) return 'iClock / ADMS'
    if (s === '5' || s.includes('zknet')) return 'ZKNET'
    return String(h)
}
function handlerTone(h) {
    const l = handlerLabel(h)
    if (l.includes('iClock')) return 'bg-emerald-50 border-emerald-200 text-emerald-700'
    if (l.includes('ZKNET')) return 'bg-amber-50 border-amber-200 text-amber-700'
    return 'bg-white border-slate-200 text-slate-600'
}
function tzLabel(tz) {
    const n = Number(tz)
    if (Number.isNaN(n)) return String(tz)
    if (n === 7) return 'WIB (GMT+7)'
    if (n === 8) return 'WITA (GMT+8)'
    if (n === 9) return 'WIT (GMT+9)'
    const sign = n >= 0 ? '+' : ''
    return `GMT${sign}${n}`
}
function lastSyncText(v) {
    if (!v || v === 0 || v === '0') return '—'
    try { return format.stamp_to_naive(v) } catch { return String(v) }
}

onMounted(() => initListController());
defineExpose({ reload: initListController })
</script>

<template>
    <div class="flex flex-col">
        <!-- Toolbar — glass pill -->
        <div
            class="px-3 sm:px-4 py-3 bg-gradient-to-r from-white to-emerald-50/30 border-b border-emerald-100 flex flex-col gap-2.5">
            <div class="flex flex-col sm:flex-row gap-2 sm:items-center">
                <div class="relative flex-1 max-w-xl">
                    <input v-model="search" type="text" placeholder="Search name, SN, location…"
                        class="w-full pl-9 pr-9 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/35 transition shadow-sm" />
                    <FontAwesome :icon="faMagnifyingGlass"
                        class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-emerald-400" />
                    <button v-if="search" @click="search = ''"
                        class="absolute right-2 top-1/2 -translate-y-1/2 w-7 h-7 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 text-xs flex items-center justify-center hover:bg-emerald-100">×</button>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                    <select v-model="statusFilter"
                        class="px-3 h-11 rounded-xl border-2 border-emerald-100 bg-white text-xs font-semibold text-green-900 outline-none focus:border-emerald-300 min-w-[140px]">
                        <option value="all">All statuses</option>
                        <option value="online">● Online</option>
                        <option value="offline">● Offline</option>
                        <option value="disable">● Disabled</option>
                    </select>
                    <button v-if="hasFilter" @click="clearFilter()"
                        class="px-3 py-2 rounded-xl bg-white border border-emerald-100 mono text-[11px] font-bold text-emerald-700 hover:bg-emerald-50">Reset</button>
                    <button @click="initListController()" title="Reload" :disabled="loading"
                        class="w-9 h-9 rounded-xl bg-emerald-600 text-white hover:bg-emerald-700 flex items-center justify-center disabled:opacity-60 shrink-0 shadow-sm">
                        <FontAwesome :icon="faRotate" class="w-3.5 h-3.5" :class="loading ? 'animate-spin' : ''" />
                    </button>
                </div>
            </div>
            <div class="flex flex-wrap items-center gap-2 mono text-[10px]">
                <span
                    class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-600 text-white font-bold">
                    <span class="w-1.5 h-1.5 rounded-full bg-white animate-pulse"></span> {{ filtered.length }} / {{
                    contents.length }} device
                </span>
                <span v-if="hasFilter"
                    class="px-2 py-1 rounded-full bg-amber-50 border border-amber-200 text-amber-700 font-bold">Active
                    filters</span>
            </div>
        </div>

        <!-- Loading skeleton -->
        <div v-if="loading" class="p-3 space-y-2 bg-white">
            <div v-for="i in 3" :key="i" class="h-14 rounded-xl bg-emerald-50 animate-pulse border border-emerald-100">
            </div>
        </div>

        <!-- Desktop table -->
        <div v-else class="hidden md:block overflow-auto max-h-[460px] bg-white">
            <table class="w-full">
                <thead class="sticky top-0 z-10">
                    <tr class="bg-gradient-to-r from-emerald-600 via-emerald-600 to-green-600 text-white">
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15">
                            <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg"
                                    class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor"
                                    stroke-width="2">
                                    <rect x="3" y="4" width="18" height="6" rx="2" />
                                    <rect x="3" y="14" width="18" height="6" rx="2" />
                                    <path d="M7 8h.01M7 18h.01" />
                                </svg> Device</span>
                        </th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15">
                            <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg"
                                    class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor"
                                    stroke-width="1.8">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M7.864 4.243A7.5 7.5 0 0119.5 10.5v2.25a7.5 7.5 0 01-7.5 7.5 7.5 7.5 0 01-7.5-7.5v-1.5a7.5 7.5 0 013.636-6.407z" />
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M12 12.75a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" />
                                </svg> Serial & Handler</span>
                        </th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 hidden lg:table-cell">
                            <span class="inline-flex items-center gap-1.5"><svg xmlns="http://www.w3.org/2000/svg"
                                    class="w-3.5 h-3.5 opacity-90" fill="none" viewBox="0 0 24 24" stroke="currentColor"
                                    stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z" />
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z" />
                                </svg> Location</span>
                        </th>
                        <th
                            class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15">
                            <span class="inline-flex items-center gap-1.5 justify-center w-full"><svg
                                    xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none"
                                    viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg> Status</span>
                        </th>
                        <th
                            class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap border-r border-white/15 hidden xl:table-cell">
                            <span class="inline-flex items-center gap-1.5 justify-center w-full"><svg
                                    xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none"
                                    viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg> Last Sync</span>
                        </th>
                        <th
                            class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase whitespace-nowrap">
                            <span class="inline-flex items-center gap-1.5 justify-center w-full"><svg
                                    xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 opacity-90" fill="none"
                                    viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                    <path stroke-linecap="round" stroke-linejoin="round"
                                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                                </svg> Actions</span>
                        </th>
                    </tr>
                </thead>
                <tbody v-if="filtered.length > 0" class="divide-y divide-emerald-50">
                    <tr v-for="device in filtered" :key="device.id"
                        class="hover:bg-emerald-50/50 transition text-xs group">
                        <td class="px-3 py-3">
                            <div class="flex items-center gap-2.5">
                                <span
                                    class="w-9 h-9 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white flex items-center justify-center shrink-0 shadow-sm">
                                    <FontAwesome :icon="faServer" class="w-3.5 h-3.5" />
                                </span>
                                <div class="min-w-0">
                                    <p class="font-bold text-green-900 leading-none truncate max-w-[16rem]">{{
                                        device.name || '—' }}</p>
                                    <p class="mono text-[10px] text-emerald-700/60 mt-0.5">ID #{{ device.id }} • {{
                                        tzLabel(device.timezone) }}</p>
                                </div>
                            </div>
                        </td>
                        <td class="px-3 py-2.5">
                            <div class="flex flex-col gap-1 items-start">
                                <span
                                    class="mono text-xs font-bold text-green-900 bg-emerald-50 border border-emerald-100 rounded-full px-2.5 py-0.5">{{
                                    device.sn }}</span>
                                <span class="inline-flex px-2 py-0.5 rounded-full border mono text-[10px] font-bold"
                                    :class="handlerTone(device.handler)">{{ handlerLabel(device.handler) }}</span>
                            </div>
                        </td>
                        <td class="px-3 py-3 hidden lg:table-cell max-w-[14rem]">
                            <p class="text-xs text-slate-600 truncate" :title="device.location">{{ device.location ||
                                '—' }}</p>
                            <p v-if="device.location" class="mono text-[10px] text-slate-400 truncate">{{
                                tzLabel(device.timezone) }}</p>
                        </td>
                        <td class="px-3 py-3 text-center">
                            <span
                                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full border mono text-[10px] font-bold shadow-sm"
                                :class="statusMeta(device.status).cls">
                                <span class="w-1.5 h-1.5 rounded-full" :class="statusMeta(device.status).dot"></span>
                                {{ statusMeta(device.status).label }}
                            </span>
                        </td>
                        <td class="px-3 py-3 text-center hidden xl:table-cell mono text-[11px] text-slate-500">
                            {{ lastSyncText(device.last_sync) }}
                        </td>
                        <td class="px-3 py-3 text-center">
                            <div class="inline-flex items-center gap-1">
                                <button @click="callbackEdit(device.id)"
                                    class="w-8 h-8 rounded-xl bg-white border-2 border-emerald-100 text-emerald-600 hover:bg-emerald-600 hover:text-white hover:border-emerald-600 flex items-center justify-center transition shadow-sm"
                                    title="Edit">
                                    <FontAwesome :icon="faPen" class="w-3 h-3" />
                                </button>
                                <button @click="callbackDelete(device.id)"
                                    class="w-8 h-8 rounded-xl bg-white border-2 border-red-100 text-red-500 hover:bg-red-500 hover:text-white hover:border-red-500 flex items-center justify-center transition shadow-sm"
                                    title="Hapus">
                                    <FontAwesome :icon="faTrash" class="w-3 h-3" />
                                </button>
                            </div>
                        </td>
                    </tr>
                </tbody>
                <tbody v-else>
                    <tr>
                        <td colspan="6" class="py-10 text-center">
                            <div
                                class="inline-flex flex-col items-center gap-2.5 px-6 py-5 rounded-2xl border-2 border-dashed border-emerald-200 bg-emerald-50/40">
                                <span
                                    class="w-12 h-12 rounded-xl bg-white border border-emerald-100 flex items-center justify-center text-emerald-600 shadow-sm">
                                    <FontAwesome :icon="faServer" class="w-5 h-5" />
                                </span>
                                <p class="mono text-xs font-bold text-emerald-700">{{ hasFilter ? 'No results' : 'No
                                    controllers yet' }}</p>
                                <p class="mono text-[11px] text-emerald-700/50 max-w-[22rem]">{{ hasFilter ? 'Try a
                                    different keyword, or reset the filter' : 'Add your first device in the form — SN
                                    and timezone is all it takes' }}</p>
                                <button v-if="hasFilter" @click="clearFilter()"
                                    class="mt-1 px-4 py-1.5 rounded-full bg-emerald-600 text-white mono text-xs font-bold">Reset
                                    filters</button>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- Mobile cards -->
        <div v-if="!loading" class="md:hidden p-2.5 space-y-2.5 max-h-[460px] overflow-auto bg-emerald-50/20">
            <div v-if="filtered.length === 0"
                class="py-8 text-center rounded-xl bg-white border-2 border-dashed border-emerald-200 mono text-xs text-emerald-700/60">
                No devices</div>
            <div v-for="device in filtered" :key="device.id" class="rounded-2xl border-2 bg-white p-3.5 shadow-sm"
                :class="Number(device.status) === 1 ? 'border-emerald-200' : 'border-slate-200'">
                <div class="flex items-start justify-between gap-2">
                    <div class="flex items-center gap-2.5 min-w-0">
                        <span
                            class="w-9 h-9 rounded-xl bg-emerald-600 text-white flex items-center justify-center shrink-0">
                            <FontAwesome :icon="faServer" class="w-3.5 h-3.5" />
                        </span>
                        <div class="min-w-0">
                            <p class="font-bold text-sm text-green-900 truncate">{{ device.name || '—' }}</p>
                            <p class="mono text-[11px] text-slate-500 truncate">SN {{ device.sn }} • {{
                                tzLabel(device.timezone) }}</p>
                        </div>
                    </div>
                    <span
                        class="shrink-0 inline-flex items-center gap-1 px-2.5 py-1 rounded-full border mono text-[10px] font-bold"
                        :class="statusMeta(device.status).cls"><span class="w-1 h-1 rounded-full bg-white"></span>{{
                            statusMeta(device.status).label }}</span>
                </div>
                <div class="mt-2.5 flex flex-wrap gap-1.5 mono text-[10px]">
                    <span class="px-2.5 py-1 rounded-full border font-bold" :class="handlerTone(device.handler)">{{
                        handlerLabel(device.handler) }}</span>
                    <span
                        class="px-2.5 py-1 rounded-full bg-white border border-slate-200 text-slate-600 truncate max-w-[14rem]">{{
                            device.location || 'No location' }}</span>
                </div>
                <p class="mt-2 mono text-[10px] text-slate-400">Sync {{ lastSyncText(device.last_sync) }} • ID #{{
                    device.id }}</p>
                <div class="mt-2.5 flex gap-2">
                    <button @click="callbackEdit(device.id)"
                        class="flex-1 py-2.5 rounded-xl bg-emerald-600 text-white mono text-xs font-bold shadow-sm">Edit</button>
                    <button @click="callbackDelete(device.id)"
                        class="flex-1 py-2.5 rounded-xl bg-white border-2 border-red-100 text-red-600 mono text-xs font-bold">Delete</button>
                </div>
            </div>
        </div>

        <div
            class="px-3 py-2.5 bg-emerald-50/40 border-t border-emerald-100 mono text-[10px] tracking-wide text-emerald-700/50 flex flex-wrap items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> {{ contents.length }} registered
            • {{ filtered.length }} shown
            <span v-if="hasFilter"
                class="ml-auto inline-flex items-center gap-1 px-2 py-0.5 rounded-full bg-amber-50 border border-amber-200 text-amber-700 font-bold"><span
                    class="w-1 h-1 rounded-full bg-amber-500"></span> active filters</span>
        </div>
    </div>
</template>
