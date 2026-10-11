<script setup>
import { useReportStore } from '~/store/report';
import { useAuthStore } from '~/store/auth';
import { useSettingsStore } from '~/store/settings';
import { useChecker, useFormater } from '#imports';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useReportStore(),
    check = useChecker(),
    format = useFormater(),
    filter = ref(""),
    deptFilter = ref(""),
    statusFilter = ref(""),
    old_range = ref(""),
    loading = ref(false),
    rangeLabel = ref($t('report.rangeToday'));

const settings = useSettingsStore()
const overtimeTol = computed(() => Number(settings.overtimeTolSec) || 0)

useHead({ title: "LuLu - Report" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

function statusTagsOf(item) {
    const tags = []
    const isAbsen = check.isNull(item.start) && check.isNull(item.end)
    if (item.late_time > 0) tags.push('late')
    // Early/overtime are checkout-based: compare actual OUT against shift end
    // in the device's own timezone (not working_time vs duration).
    if (!isAbsen && !check.isNull(item.end)) {
        const tz = Number(item.tz) || 0
        const endRef = (Number(item.date) || 0) - tz * 3600 + (item.shift_end ?? 0)
        // Tolerance comes from the backend (the `settings` table); the early/overtime
        // tags must use the same threshold as the `overtime` the backend sends.
        if (Number(item.end) < endRef - overtimeTol.value) tags.push('early')
    }
    if (Number(item.overtime) > 0) tags.push('overtime')
    // Counted on an included holiday: real work at a different pay rate, so payroll
    // must be able to pick these rows out. Kept separate from late/overtime because it
    // describes the DAY, not this person's punctuality.
    if (item.holiday_name) tags.push('holiday')
    if (!isAbsen && check.isNull(item.end)) tags.push('early')
    if (isAbsen) tags.push('puncht')
    if (tags.length === 0 && !isAbsen) tags.push('present')
    return tags
}
function tzClock(ts, tz) {
    if (check.isNull(ts)) return '-'
    const n = Number(tz)
    // Device TZ render: enroll timestamps are UTC epochs (same as LiveTable).
    if (!Number.isNaN(n)) {
        const d = new Date((Number(ts) + n * 3600) * 1000)
        const p = (v) => String(v).padStart(2, '0')
        return `${p(d.getUTCHours())}:${p(d.getUTCMinutes())}:${p(d.getUTCSeconds())}`
    }
    return format.stamp_to_naive_time(ts, false)
}

const allCount = computed(() => (list.contents || []).length)
const depts = computed(() => {
    const s = new Set((list.contents || []).map(x => x.departement).filter(Boolean))
    return [...s].sort()
})
const filteredPreview = computed(() => {
    let arr = list.contents || []
    const q = (filter.value || '').trim().toLowerCase()
    if (q) arr = arr.filter(item => (`${item.first_name || ''} ${item.last_name || ''} ${item.departement || ''}`).toLowerCase().includes(q))
    if (deptFilter.value) arr = arr.filter(item => (item.departement || '') === deptFilter.value)
    if (statusFilter.value) arr = arr.filter(item => statusTagsOf(item).includes(statusFilter.value))
    return arr
})
const filteredCount = computed(() => filteredPreview.value.length)

// stats from filtered
const statPresent = computed(() => filteredPreview.value.filter(x => statusTagsOf(x).includes('present')).length)
const statLate = computed(() => filteredPreview.value.filter(x => statusTagsOf(x).includes('late')).length)
const statAbsent = computed(() => filteredPreview.value.filter(x => statusTagsOf(x).includes('puncht')).length)
const statEarly = computed(() => filteredPreview.value.filter(x => statusTagsOf(x).includes('early')).length)
const statOvertime = computed(() => filteredPreview.value.filter(x => statusTagsOf(x).includes('overtime')).length)
const totalWorkingSec = computed(() => filteredPreview.value.reduce((a, c) => a + (c.working_time || 0), 0))
function fmtHMS(sec) {
    if (!sec) return '0h 0m'
    const h = Math.floor(sec / 3600)
    const m = Math.floor((sec % 3600) / 60)
    return `${h}h ${m}m`
}

async function fetchRange(startTs, endTs, label, opts = {}) {
    const new_range = String(startTs) + String(endTs)
    const isSame = new_range === old_range.value
    // allow force refetch from picker even if same, but debounce preset+picker double on mount
    if (isSame && !opts.force) {
        // if already has data for this range, skip duplicate; if empty, allow retry (fixes post-backend-fix stale empty)
        if ((list.contents || []).length) return
    }
    // dedupe rapid double-call (picker emit + preset both today) within 800ms
    if (isSame && opts.force && Date.now() - (fetchRange._last || 0) < 800) return
    loading.value = true
    try {
        const resp = await apiFetch(`${config.public.apiBase}/report/range/${startTs}/${endTs}`, { method: "GET", headers: auth.confHeaders() })
        if (resp.code == 200) {
            list.set(resp.data || [])
            old_range.value = new_range
            fetchRange._last = Date.now()
            if (label) rangeLabel.value = label
        } else {
            console.warn('[report] non-200', resp)
        }
    } catch (e) {
        console.error('[report] fetch failed', e)
    }
    loading.value = false
}

async function getDataReport(ev) {
    // from SimpleDatePicker - force refetch even if same (user explicitly picked/edited)
    if (!ev || ev.startTimestamp == null || ev.endTimestamp == null) return
    await fetchRange(ev.startTimestamp, ev.endTimestamp, `${ev.startDate} → ${ev.endDate}`, { force: true })
}

function toUnix(dateObj) {
    const tz = new Date().getTimezoneOffset() * 60 * -1
    return Math.floor(dateObj.getTime() / 1000) + tz
}
function startOfDay(d) { const x = new Date(d); x.setHours(0, 0, 0, 0); return x }
function endOfDay(d) { const x = new Date(d); x.setHours(0, 0, 0, 0); return x }

async function preset(kind) {
    const now = new Date()
    let s, e, label
    if (kind === 'today') {
        s = startOfDay(now); e = endOfDay(now); label = $t('report.rangeToday')
    } else if (kind === 'yesterday') {
        const y = new Date(now); y.setDate(y.getDate() - 1)
        s = startOfDay(y); e = endOfDay(y); label = $t('report.rangeYesterday')
    } else if (kind === '7d') {
        e = endOfDay(now); s = startOfDay(new Date(now.getTime() - 6 * 86400000)); label = $t('report.range7long')
    } else if (kind === '30d') {
        e = endOfDay(now); s = startOfDay(new Date(now.getTime() - 29 * 86400000)); label = $t('report.range30long')
    } else if (kind === 'month') {
        s = startOfDay(new Date(now.getFullYear(), now.getMonth(), 1)); e = endOfDay(now); label = $t('report.rangeMonth')
    } else return
    await fetchRange(toUnix(s), toUnix(e), label)
}

function exportCsv() {
    const rows = filteredPreview.value
    if (!rows.length) return
    const header = ['Employee', 'Department', 'Date', 'Shift', 'Schedule', 'In', 'Out', 'Status', 'Holiday', 'Worked', 'Late']
    function esc(v) { const s = String(v ?? '').replace(/"/g, '""'); return `"${s}"` }
    function rowTags(it) {
        const t = statusTagsOf(it)
        const map = { present: 'Present', late: 'Late', early: 'Early Leave', puncht: 'Absent', overtime: 'Overtime', holiday: 'Holiday' }
        return t.map(x => map[x]).join(' | ')
    }
    const lines = [header.map(esc).join(',')]
    for (const it of rows) {
        lines.push([
            `${it.first_name || ''} ${it.last_name || ''}`.trim(),
            it.departement || '',
            format.stamp_to_naive_date(it.date),
            it.shift_name || '',
            it.schedule_name || '',
            check.isNull(it.start) ? '-' : tzClock(it.start, it.tz),
            check.isNull(it.end) ? '-' : tzClock(it.end, it.tz),
            rowTags(it),
            it.holiday_name || '',
            format.sec_to_naive(it.working_time || 0),
            it.late_time > 0 ? format.sec_to_naive(it.late_time) : '-'
        ].map(esc).join(','))
    }
    const blob = new Blob([lines.join('\r\n')], { type: 'text/csv;charset=utf-8;' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = `laporan-${rangeLabel.value.replace(/\s+/g, '_')}-${format.stamp_to_naive_date(Date.now() / 1000).replace(/-/g, '')}.csv`
    document.body.appendChild(a); a.click(); a.remove(); URL.revokeObjectURL(url)
}

function clearFilters() { filter.value = ''; deptFilter.value = ''; statusFilter.value = '' }

onMounted(async () => {
    // The backend tolerance is used to flag the overtime/early tags.
    settings.fetchSettings()
    // auto load today if empty (ListReport's initReport is disabled - we load here)
    if (!allCount.value) {
        await preset('today')
    }
    // also allow manual refresh via ListReport init? no-op
})
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
                    class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8">
                        <rect x="3" y="4" width="18" height="18" rx="2" />
                        <path d="M16 2v4M8 2v4M3 10h18" />
                        <path d="M9 16l2 2 4-4" />
                    </svg>
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">{{ $t('report.title') }}
                    </h1>
                    <div class="mt-2 flex flex-wrap items-center gap-2 mono text-[10px]">
                        <span class="chip chip-emerald">{{ rangeLabel }}</span>
                        <span class="inline-flex items-center gap-1.5 text-emerald-700/40"><span
                                class="w-1.5 h-1.5 rounded-full bg-emerald-500"
                                :class="loading ? 'animate-pulse' : ''"></span> {{ allCount }} data • {{ filteredCount
                                }}
                            shown</span>
                        <span class="hidden sm:inline-flex chip bg-white border-emerald-200 text-emerald-700 border">{{
                            $t('report.titleUpper') }}</span>
                    </div>
                </div>
            </div>
            <div class="lg:ml-auto flex flex-wrap items-center gap-2">
                <button @click="exportCsv()" :disabled="!filteredCount"
                    class="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-white border-2 border-emerald-100 text-emerald-700 mono text-xs font-bold hover:bg-emerald-50 disabled:opacity-40">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8">
                        <path d="M12 3v12" />
                        <path d="M8 11l4 4 4-4" />
                        <path d="M3 17v2a2 2 0 002 2h14a2 2 0 002-2v-2" />
                    </svg>
                    {{ $t('report.exportCsv') }}
                </button>
                <button @click="preset('today'); clearFilters()"
                    class="hidden sm:inline-flex px-3 py-2 rounded-xl bg-emerald-600 text-white mono text-xs font-bold hover:bg-emerald-700">{{
                        $t('report.loadToday') }}</button>
            </div>
        </div>

        <!-- Controls -->
        <div class="glass rounded-2xl border border-emerald-100 px-4 py-3 space-y-3">
            <!-- Row 1: date picker + presets -->
            <div class="flex flex-col lg:flex-row lg:items-center gap-3">
                <div class="flex items-center gap-2">
                    <SimpleDatePicker :use-utc="true" :init-start-date="format.stamp_to_naive_date()"
                        :init-end-date="format.stamp_to_naive_date()" v-on:date-range-selected="getDataReport" />
                    <span v-if="loading" class="inline-flex items-center gap-1.5 mono text-xs text-emerald-700"><span
                            class="w-3 h-3 border-2 border-emerald-200 border-t-emerald-600 rounded-full animate-spin"></span>
                        {{ $t('common.loading') }}</span>
                </div>
                <div class="flex flex-wrap items-center gap-1.5 lg:ml-auto">
                    <span class="mono text-[10px] tracking-widest text-emerald-700/40 hidden sm:inline">{{
                        $t('report.quick') }}</span>
                    <button @click="preset('today')" class="px-2.5 py-1.5 rounded-full border text-xs font-bold mono"
                        :class="rangeLabel === $t('report.rangeToday') ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-100 text-emerald-700 hover:bg-emerald-50'">{{
                            $t('report.rangeToday') }}</button>
                    <button @click="preset('yesterday')"
                        class="px-2.5 py-1.5 rounded-full border text-xs font-bold mono"
                        :class="rangeLabel === $t('report.rangeYesterday') ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-100 text-emerald-700 hover:bg-emerald-50'">{{
                            $t('report.rangeYesterday') }}</button>
                    <button @click="preset('7d')" class="px-2.5 py-1.5 rounded-full border text-xs font-bold mono"
                        :class="rangeLabel === $t('report.range7long') ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-100 text-emerald-700 hover:bg-emerald-50'">{{
                            $t('report.range7') }}</button>
                    <button @click="preset('30d')" class="px-2.5 py-1.5 rounded-full border text-xs font-bold mono"
                        :class="rangeLabel === $t('report.range30long') ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-100 text-emerald-700 hover:bg-emerald-50'">{{
                            $t('report.range30') }}</button>
                    <button @click="preset('month')" class="px-2.5 py-1.5 rounded-full border text-xs font-bold mono"
                        :class="rangeLabel === $t('report.rangeMonth') ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white border-emerald-100 text-emerald-700 hover:bg-emerald-50'">{{
                            $t('report.rangeMonth') }}</button>
                </div>
            </div>
            <!-- Row 2: search + dept + status -->
            <div class="grid grid-cols-1 lg:grid-cols-12 gap-3">
                <div class="lg:col-span-5 relative">
                    <input v-model="filter" type="text" :placeholder="$t('report.searchPh')"
                        class="w-full pl-9 pr-3 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30">
                    <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-emerald-700/40"
                        viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8">
                        <path d="M21 21l-4.2-4.2" />
                        <circle cx="11" cy="11" r="6" />
                    </svg>
                    <button v-if="filter" @click="filter = ''"
                        class="absolute right-2 top-1/2 -translate-y-1/2 w-7 h-7 rounded-full bg-emerald-50 text-emerald-700 flex items-center justify-center text-xs">×</button>
                </div>
                <div class="lg:col-span-3">
                    <select v-model="deptFilter"
                        class="w-full px-3 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">{{ $t('report.allDepartments') }}</option>
                        <option v-for="d in depts" :key="d" :value="d">{{ d }}</option>
                    </select>
                </div>
                <div class="lg:col-span-3">
                    <select v-model="statusFilter"
                        class="w-full px-3 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                        <option value="">{{ $t('report.allStatuses') }}</option>
                        <option value="present">{{ $t('report.statusPresent') }}</option>
                        <option value="late">{{ $t('report.statusLate') }}</option>
                        <option value="early">{{ $t('report.statusEarlyLeave') }}</option>
                        <option value="puncht">{{ $t('report.statusAbsent') }}</option>
                        <option value="overtime">{{ $t('report.statusOvertime') }}</option>
                        <option value="holiday">{{ $t('report.statusHoliday') }}</option>
                    </select>
                </div>
                <div class="lg:col-span-1 flex">
                    <button @click="clearFilters()"
                        class="w-full px-3 h-11 rounded-xl bg-white border-2 border-slate-200 text-slate-700 text-sm font-medium hover:bg-slate-50">{{
                            $t('common.reset') }}</button>
                </div>
            </div>
            <div v-if="filter || deptFilter || statusFilter"
                class="flex flex-wrap items-center gap-1.5 mono text-[11px]">
                <span class="text-emerald-700/50">{{ $t('common.activeFilters') }}</span>
                <span v-if="filter"
                    class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700">Search: "{{
                        filter }}"</span>
                <span v-if="deptFilter"
                    class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{ deptFilter
                    }}</span>
                <span v-if="statusFilter"
                    class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700">{{
                        {
                            present: 'Present', late: 'Late', early:
                                'Early Leave', puncht: 'Absent', overtime: 'Overtime', holiday: 'Holiday'
                        }[statusFilter] }}</span>
                <span class="text-slate-400">- {{ filteredCount }} of {{ allCount }} rows</span>
            </div>
        </div>

        <!-- Stats -->
        <div class="grid grid-cols-2 lg:grid-cols-6 gap-3">
            <div class="rounded-2xl bg-white border border-emerald-100 p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-emerald-700/60">{{ $t('report.statPresent') }}</p>
                <p class="text-xl font-extrabold text-green-900">{{ statPresent }}</p>
            </div>
            <div class="rounded-2xl bg-emerald-600 text-white p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-white/70">{{ $t('report.statOnTime') }}</p>
                <p class="text-xl font-extrabold">{{ statPresent }}</p>
            </div>
            <div class="rounded-2xl bg-white border border-red-100 p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-red-600/70">{{ $t('report.statLate') }}</p>
                <p class="text-xl font-extrabold text-red-600">{{ statLate }}</p>
            </div>
            <div class="rounded-2xl bg-white border border-slate-200 p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-slate-500">{{ $t('report.statAbsent') }}</p>
                <p class="text-xl font-extrabold text-slate-800">{{ statAbsent }}</p>
            </div>
            <div class="rounded-2xl bg-white border border-amber-100 p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-amber-600/70">{{ $t('report.statEarly') }}</p>
                <p class="text-xl font-extrabold text-amber-600">{{ statEarly }}</p>
            </div>
            <div class="rounded-2xl bg-white border border-emerald-100 p-3.5">
                <p class="mono text-[10px] tracking-[0.14em] text-emerald-700/60">{{ $t('report.statTotalHours') }}</p>
                <p class="text-xl font-extrabold text-green-900">{{ fmtHMS(totalWorkingSec) }}</p>
            </div>
        </div>

        <!-- Table card -->
        <div class="hud-frame glass rounded-2xl border border-emerald-100 overflow-hidden">
            <div class="px-4 py-2.5 bg-white/60 border-b border-emerald-100 flex items-center justify-between gap-2">
                <h2 class="mono text-xs font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                    <span class="w-1.5 h-1.5 rounded-full bg-emerald-500" :class="loading ? 'animate-ping' : ''"></span>
                    {{ $t('report.dailySummary') }}
                </h2>
                <span class="mono text-[11px] text-emerald-700/60 shrink-0">{{ filteredCount }} of {{ allCount }}
                    rows</span>
            </div>
            <ListReport :filter="filter" :dept-filter="deptFilter" :status-filter="statusFilter" />
        </div>

        <p class="mono text-[10px] tracking-wide text-emerald-700/40 px-1">{{ $t('report.activeRange') }} <b
                class="text-emerald-700">{{ rangeLabel }}</b></p>
    </div>
</template>
