<script setup>
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';

const auth = useAuthStore(),
    config = useRuntimeConfig(),
    notify = useNotifyStore(),
    data = ref([]),
    loading = ref(false),
    error = ref('');

// The unregistered list is a to-do list, not just a report: every row is a person
// missing from the employee table. Clicking a row hands that exact number to the
// employee form so the operator only has to type a name, never look it up.
const emit = defineEmits(['link'])

function tzShort(tz) {
    const n = Number(tz);
    if (Number.isNaN(n)) return '';
    return `GMT${n >= 0 ? '+' : ''}${n}`;
}

function formatTime(ts, tz) {
    const n = Number(tz);
    if (Number.isNaN(n) || !ts) return '-';
    const utc = new Date(ts * 1000);
    const offset = n * 3600000;
    const local = new Date(utc.getTime() + offset);
    const y = local.getUTCFullYear();
    const m = String(local.getUTCMonth() + 1).padStart(2, '0');
    const d = String(local.getUTCDate()).padStart(2, '0');
    const h = String(local.getUTCHours()).padStart(2, '0');
    const mi = String(local.getUTCMinutes()).padStart(2, '0');
    return `${y}-${m}-${d} ${h}:${mi}`;
}

async function init() {
    loading.value = true;
    error.value = '';
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/unregistered`, {
            method: 'GET',
            headers: auth.confHeaders(),
        });
        if (r.code === 200) {
            data.value = r.data || [];
        } else {
            error.value = r.message || 'Failed to load';
        }
    } catch (e) {
        error.value = 'Connection failed';
    }
    loading.value = false;
}

function reload() { init(); }
defineExpose({ reload, init });
onMounted(() => { init(); });
</script>

<template>
    <div class="flex flex-col overflow-hidden">
        <div
            class="px-4 py-2.5 bg-gradient-to-r from-emerald-50/70 to-white border-b border-emerald-100 flex items-center justify-between gap-2">
            <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                <span class="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse"></span>
                {{ $t('employee.unregTitle') }}
            </span>
            <span class="mono text-[11px] text-emerald-700/60">{{ data.length }} {{ data.length === 1 ? 'entry' :
                'entries' }}</span>
            <button @click="reload()" :title="$t('device.reloadShort')"
                class="h-8 px-2.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 inline-flex items-center justify-center gap-1.5 mono text-[11px] font-bold shrink-0 shadow-sm">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2">
                    <path d="M21 12a9 9 0 11-2.64-6.36" />
                    <path d="M21 3v6h-6" />
                </svg>
                <span class="hidden sm:inline">{{ $t('device.reloadShort') }}</span>
            </button>
        </div>

        <div v-if="loading" class="px-4 py-12 text-center text-slate-500">{{ $t('common.loading') }}</div>
        <div v-else-if="error" class="px-4 py-12 text-center text-red-600">{{ error }}</div>
        <div v-else-if="data.length === 0" class="px-4 py-14 text-center">
            <div class="flex flex-col items-center gap-3">
                <div
                    class="w-12 h-12 rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600">
                    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8">
                        <path d="M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2" />
                        <circle cx="9" cy="7" r="4" />
                        <path d="M18 8a2.5 2.5 0 010 5" />
                        <path d="M20 11a4 4 0 01-1 2.5" />
                    </svg>
                </div>
                <p class="mono text-xs font-bold tracking-[0.14em] text-emerald-700">{{ $t('employee.unregEmpty') }}</p>
                <p class="text-sm text-slate-500">{{ $t('employee.unregNone') }}</p>
            </div>
        </div>

        <div v-else class="overflow-auto max-h-[52vh] lg:max-h-[56vh] hidden md:block scrollbar-thin">
            <table class="w-full min-w-[640px] border-collapse">
                <thead class="sticky top-0 z-10">
                    <tr class="bg-gradient-to-r from-amber-600 to-amber-700 text-white">
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[15%]">
                            {{ $t('employee.unregColUid') }}</th>
                        <th
                            class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[15%]">
                            {{ $t('employee.unregColPunches') }}</th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 w-[35%]">
                            {{ $t('employee.unregColFirst') }}</th>
                        <th
                            class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase w-[35%]">
                            {{ $t('employee.unregColLast') }}</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-amber-50 bg-white">
                    <tr v-for="(row, idx) in data" :key="`unreg-${row.device_uid}`"
                        class="hover:bg-amber-50/60 transition" :class="idx % 2 === 0 ? 'bg-white' : 'bg-amber-50/20'">
                        <td class="py-3 px-3">
                            <span
                                class="inline-flex px-2.5 py-1 rounded-full bg-amber-100 border border-amber-300 text-amber-800 mono text-[11px] font-bold">
                                #{{ row.device_uid }}
                            </span>
                            <span v-if="tzShort(row.timezone)" class="ml-1.5 mono text-[10px] text-slate-400">{{
                                tzShort(row.timezone) }}</span>
                        </td>
                        <td class="py-3 px-3 text-center">
                            <span class="mono text-[12px] font-bold text-slate-700">{{ row.punch_count }}</span>
                        </td>
                        <td class="py-3 px-3">
                            <span class="mono text-[10px] text-slate-600">{{ formatTime(row.first_punch, row.timezone)
                                }}</span>
                        </td>
                        <td class="py-3 px-3 flex items-center justify-between gap-2">
                            <span class="mono text-[10px] text-slate-600">{{ formatTime(row.last_punch, row.timezone)
                                }}</span>
                            <button @click="emit('link', row.device_uid)"
                                class="h-7 px-2.5 rounded-lg bg-emerald-600 text-white hover:bg-emerald-700 text-xs font-bold shrink-0 transition shadow-sm">Register</button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <div v-if="data.length > 0" class="md:hidden p-3 space-y-3 max-h-[52vh] overflow-auto">
            <div v-for="row in data" :key="`unreg-m-${row.device_uid}`"
                class="rounded-2xl border border-amber-100 bg-white p-3.5 shadow-sm">
                <div class="flex items-start justify-between gap-3">
                    <div>
                        <p class="text-sm font-bold text-slate-900">User ID #{{ row.device_uid }}</p>
                        <p class="mono text-[11px] text-amber-600 mt-1">{{ row.punch_count }} punches</p>
                    </div>
                    <button @click="emit('link', row.device_uid)"
                        class="h-8 px-3 rounded-lg bg-emerald-600 text-white text-xs font-bold shrink-0 shadow-sm">Register</button>
                </div>
                <div class="mt-3 space-y-1">
                    <p class="mono text-[10px] text-slate-600"><span class="text-slate-400">First:</span> {{
                        formatTime(row.first_punch, row.timezone) }}</p>
                    <p class="mono text-[10px] text-slate-600"><span class="text-slate-400">Last:</span> {{
                        formatTime(row.last_punch, row.timezone) }}</p>
                </div>
            </div>
        </div>

        <div v-if="data.length > 0"
            class="px-4 py-2.5 bg-white/90 backdrop-blur border-t border-amber-100 flex items-center justify-between mono text-[10px]">
            <span class="text-amber-700/60">{{ $t('employee.unregFooter') }}</span>
        </div>
    </div>
</template>
