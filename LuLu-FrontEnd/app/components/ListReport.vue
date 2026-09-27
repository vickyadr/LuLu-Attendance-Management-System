<script setup>
import { useReportStore } from '~/store/report';
import { useChecker, useFormater } from '#imports';

const
    list = useReportStore(),
    check = useChecker(),
    format = useFormater(),
    props = defineProps({
        filter: { type: String, default: '' },
        deptFilter: { type: String, default: '' },
        statusFilter: { type: String, default: '' },
    });

function statusTags(issue) {
    const tags = []
    const isAbsen = check.isNull(issue.start) && check.isNull(issue.end)
    if (issue.late_time > 0) tags.push('late')

    if (!isAbsen && !check.isNull(issue.end)) {
        const tz = Number(issue.tz) || 0
        const endRef = (Number(issue.date) || 0) - tz * 3600 + (issue.shift_end ?? 0)
        if (Number(issue.end) < endRef - 60) tags.push('early')
        if (Number(issue.end) > endRef + 60) tags.push('overtime')
    }
    if (!isAbsen && check.isNull(issue.end)) tags.push('early')
    if (isAbsen) tags.push('puncht')
    if (tags.length === 0 && !isAbsen) tags.push('present')
    return tags
}
function hasTag(issue, tag) { return statusTags(issue).includes(tag) }

const contents = computed(() => {
    let arr = list.contents || []
    const q = (props.filter || '').trim().toLowerCase()
    if (q) arr = arr.filter(item => (`${item.first_name || ''} ${item.last_name || ''} ${item.departement || ''}`).toLowerCase().includes(q))
    if (props.deptFilter) arr = arr.filter(item => (item.departement || '') === props.deptFilter)
    if (props.statusFilter) {
        arr = arr.filter(item => hasTag(item, props.statusFilter))
    }
    return arr
});

function initials(item) {
    const a = (item.first_name || '')[0] || '?'
    const b = (item.last_name || '')[0] || ''
    return (a + b).toUpperCase().slice(0, 2)
}
function avatarBg(item) {
    const s = `${item.first_name || ''}${item.last_name || ''}`
    let h = 0; for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) % 360
    // emerald hues but varied
    return `hsl(${140 + (h % 40) - 20} 70% 38%)`
}
function fmtDate(ts) { return format.stamp_to_naive_date(ts) }
function fmtTime(ts, tz) {
    if (check.isNull(ts)) return '—'
    const n = Number(tz)
    // Device TZ render: enroll timestamps are UTC epochs, shift by the
    // device's own timezone and read back with UTC getters (like LiveTable).
    if (!Number.isNaN(n)) {
        const d = new Date((Number(ts) + n * 3600) * 1000)
        const p = (v) => String(v).padStart(2, '0')
        return `${p(d.getUTCHours())}:${p(d.getUTCMinutes())}:${p(d.getUTCSeconds())}`
    }
    return format.stamp_to_naive_time(ts, false)
}
function tzShort(tz) {
    const n = Number(tz)
    if (Number.isNaN(n)) return ''
    return `GMT${n >= 0 ? '+' : ''}${n}`
}
function fmtWday(ts) {
    try { return format.stamp_to_weekday(ts, false, true) } catch { return '' }
}
function fmtDur(sec) {
    if (check.isNull(sec)) return '—'
    return format.sec_to_naive(sec)
}
function shiftDur(issue) {
    const s = issue.shift_start ?? 0
    const e = issue.shift_end ?? 0
    return Math.max(0, e >= s ? e - s : e - s + 86400)
}
function overtimeSec(issue) {
    // Overtime amount = actual OUT minus shift end, in the device's own
    // timezone. Mirrors the overtime tag rule (>60s tolerance).
    if (check.isNull(issue.end)) return 0
    const tz = Number(issue.tz) || 0
    const endRef = (Number(issue.date) || 0) - tz * 3600 + (issue.shift_end ?? 0)
    const over = Number(issue.end) - endRef
    return over > 60 ? over : 0
}
</script>

<template>
    <div class="overflow-auto max-h-[62vh] md:max-h-[58vh]">
        <!-- TABLE desktop -->
        <table class="w-full min-w-[860px] divide-y divide-emerald-100 hidden md:table">
            <thead class="bg-emerald-50/70 sticky top-0 z-10 backdrop-blur">
                <tr class="text-left mono text-[11px] tracking-[0.12em] text-emerald-800/60">
                    <th class="py-3 px-4 font-bold">EMPLOYEES</th>
                    <th class="py-3 px-3 font-bold">DATE</th>
                    <th class="py-3 px-3 font-bold">SHIFT</th>
                    <th class="py-3 px-3 font-bold">PUNCH</th>
                    <th class="py-3 px-3 font-bold">STATUS</th>
                    <th class="py-3 px-3 font-bold text-right">DURATION</th>
                </tr>
            </thead>
            <tbody v-if="contents && contents.length" class="divide-y divide-emerald-50 bg-white">
                <tr v-for="(issue, idx) in contents" :key="`${issue.id}-${issue.date}-${idx}`"
                    class="hover:bg-emerald-50/60 transition" :class="idx % 2 === 0 ? 'bg-white' : 'bg-emerald-50/20'">
                    <!-- Employee -->
                    <td class="py-3 px-4">
                        <div class="flex items-center gap-3">
                            <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white text-xs font-extrabold shrink-0 shadow-sm"
                                :style="{ background: avatarBg(issue) }">{{ initials(issue) }}</div>
                            <div class="min-w-0">
                                <div class="text-sm font-bold text-green-900 leading-none truncate">{{ issue.first_name
                                    }} {{ issue.last_name }}</div>
                                <div class="mt-1 flex items-center gap-1.5">
                                    <span
                                        class="inline-flex px-1.5 py-0.5 rounded-full bg-emerald-50 border border-emerald-100 text-emerald-700 mono text-[10px] font-bold tracking-wide">{{
                                        issue.departement || '—' }}</span>
                                    <span class="mono text-[10px] text-slate-400">#{{ issue.id }}</span>
                                </div>
                            </div>
                        </div>
                    </td>
                    <!-- Date -->
                    <td class="py-3 px-3">
                        <div class="inline-flex flex-col items-start">
                            <span
                                class="px-2.5 py-1 rounded-full bg-white border border-emerald-100 mono text-xs font-bold text-green-900">{{
                                fmtDate(issue.date) }}</span>
                            <span class="mt-1 mono text-[10px] tracking-widest text-emerald-700/50">{{
                                fmtWday(issue.date) }}</span>
                        </div>
                    </td>
                    <!-- Shift -->
                    <td class="py-3 px-3">
                        <div class="inline-flex flex-col gap-1">
                            <span
                                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold tracking-wide">
                                {{ issue.shift_name || '—' }}
                                <span v-if="issue.schedule_name" class="hidden lg:inline opacity-80">• {{
                                    issue.schedule_name }}</span>
                            </span>
                            <span class="mono text-[11px] text-slate-600">{{ format.sec_to_naive(issue.shift_start) }} —
                                {{ format.sec_to_naive(issue.shift_end) }}</span>
                            <span class="mono text-[10px] text-emerald-700/40">Shift {{ fmtDur(shiftDur(issue))
                                }}</span>
                        </div>
                    </td>
                    <!-- Enroll In/Out -->
                    <td class="py-3 px-3">
                        <div class="space-y-1.5 min-w-[140px]">
                            <div class="flex items-center gap-2 mono text-xs">
                                <span
                                    class="w-7 mono text-[10px] font-bold tracking-widest text-emerald-700/60">IN</span>
                                <span
                                    class="inline-flex items-center gap-1.5 px-2 py-1 rounded-full border text-xs font-bold"
                                    :class="check.isNull(issue.start) ? 'bg-slate-50 border-slate-200 text-slate-400' : (issue.late_time > 0 ? 'bg-red-50 border-red-200 text-red-700' : 'bg-emerald-50 border-emerald-200 text-emerald-700')">
                                    <span class="w-1.5 h-1.5 rounded-full"
                                        :class="check.isNull(issue.start) ? 'bg-slate-300' : (issue.late_time > 0 ? 'bg-red-500' : 'bg-emerald-500')"></span>
                                    {{ fmtTime(issue.start, issue.tz) }}
                                </span>
                            </div>
                            <div class="flex items-center gap-2 mono text-xs">
                                <span
                                    class="w-7 mono text-[10px] font-bold tracking-widest text-emerald-700/60">OUT</span>
                                <span
                                    class="inline-flex items-center gap-1.5 px-2 py-1 rounded-full border text-xs font-bold"
                                    :class="check.isNull(issue.end) ? 'bg-slate-50 border-slate-200 text-slate-400' : 'bg-white border-emerald-100 text-green-900'">
                                    <span class="w-1.5 h-1.5 rounded-full"
                                        :class="check.isNull(issue.end) ? 'bg-slate-300' : 'bg-emerald-500'"></span>
                                    {{ fmtTime(issue.end, issue.tz) }}
                                </span>
                            </div>
                            <div v-if="issue.enroll_device"
                                class="mono text-[10px] text-slate-400 truncate max-w-[160px]">{{ issue.enroll_device
                                }}<span v-if="issue.enroll_location"> • {{ issue.enroll_location }}</span><span
                                    v-if="issue.tz !== undefined && issue.tz !== null"> • {{ tzShort(issue.tz) }}</span>
                            </div>
                        </div>
                    </td>
                    <!-- Tags -->
                    <td class="py-3 px-3">
                        <div class="flex flex-wrap gap-1 max-w-[180px]">
                            <span v-if="hasTag(issue, 'late')"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-red-50 border border-red-200 text-red-700 mono text-[10px] font-bold">Late</span>
                            <span v-if="hasTag(issue, 'early')"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-amber-50 border border-amber-200 text-amber-700 mono text-[10px] font-bold">Early
                                Leave</span>
                            <span v-if="hasTag(issue, 'puncht')"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-slate-800 text-white mono text-[10px] font-bold">Absent</span>
                            <span v-if="hasTag(issue, 'overtime')"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Overtime</span>
                            <span v-if="hasTag(issue, 'present')"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Present</span>
                        </div>
                    </td>
                    <!-- Details -->
                    <td class="py-3 px-3 text-right">
                        <div class="inline-flex flex-col items-end gap-1 mono text-xs">
                            <span
                                class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white border border-emerald-100 text-green-900 font-bold">
                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                                Worked {{ fmtDur(issue.working_time) }}
                            </span>
                            <span v-if="issue.late_time > 0"
                                class="px-2 py-0.5 rounded-full bg-red-50 border border-red-100 text-red-600 mono text-[10px] font-bold">Late
                                {{ fmtDur(issue.late_time) }}</span>
                            <span v-if="overtimeSec(issue) > 0"
                                class="px-2 py-0.5 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Overtime
                                {{ fmtDur(overtimeSec(issue)) }}</span>
                        </div>
                    </td>
                </tr>
            </tbody>
            <tbody v-else>
                <tr>
                    <td colspan="6" class="py-16">
                        <div class="flex flex-col items-center gap-3 text-center px-6">
                            <div
                                class="w-12 h-12 rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600">
                                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                    stroke-width="1.8">
                                    <rect x="3" y="4" width="18" height="18" rx="2" />
                                    <path d="M16 2v4M8 2v4M3 10h18" />
                                    <path d="M9 16l2 2 4-4" />
                                </svg>
                            </div>
                            <p class="mono text-xs font-bold tracking-widest text-emerald-700">NO DATA</p>
                            <p class="text-sm text-slate-500 max-w-md">Pick a date range above or change the filters.
                                Data appears once the devices sync.</p>
                        </div>
                    </td>
                </tr>
            </tbody>
        </table>

        <!-- CARD mobile -->
        <div class="md:hidden p-3 space-y-3" v-if="contents && contents.length">
            <div v-for="(issue, idx) in contents" :key="`m-${issue.id}-${issue.date}-${idx}`"
                class="rounded-2xl border border-emerald-100 bg-white p-3.5 shadow-sm">
                <div class="flex items-start justify-between gap-3">
                    <div class="flex items-center gap-2.5">
                        <div class="w-9 h-9 rounded-xl flex items-center justify-center text-white text-xs font-extrabold"
                            :style="{ background: avatarBg(issue) }">{{ initials(issue) }}</div>
                        <div>
                            <p class="text-sm font-bold text-green-900 leading-none">{{ issue.first_name }} {{
                                issue.last_name }}</p>
                            <p class="mono text-[10px] text-slate-400 mt-1">#{{ issue.id }} • {{ issue.departement ||
                                '—' }}</p>
                        </div>
                    </div>
                    <span class="px-2 py-1 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold">{{
                        issue.shift_name || '—' }}</span>
                </div>
                <div class="mt-3 grid grid-cols-2 gap-2 mono text-xs">
                    <div class="rounded-xl bg-emerald-50/60 border border-emerald-100 px-3 py-2">
                        <p class="mono text-[10px] tracking-widest text-emerald-700/50">DATE</p>
                        <p class="font-bold text-green-900">{{ fmtDate(issue.date) }}</p>
                        <p class="text-[11px] text-slate-500">{{ fmtWday(issue.date) }}</p>
                    </div>
                    <div class="rounded-xl bg-white border border-emerald-100 px-3 py-2">
                        <p class="mono text-[10px] tracking-widest text-emerald-700/50">SHIFT</p>
                        <p class="font-bold text-green-900">{{ format.sec_to_naive(issue.shift_start) }} — {{
                            format.sec_to_naive(issue.shift_end) }}</p>
                        <p class="text-[11px] text-slate-500">{{ issue.schedule_name || '' }}</p>
                    </div>
                </div>
                <div class="mt-2 flex flex-wrap gap-1.5">
                    <span class="inline-flex px-2 py-1 rounded-full border mono text-[11px] font-bold"
                        :class="check.isNull(issue.start) ? 'bg-slate-50 border-slate-200 text-slate-400' : (issue.late_time > 0 ? 'bg-red-50 border-red-200 text-red-700' : 'bg-emerald-50 border-emerald-200 text-emerald-700')">IN
                        {{ fmtTime(issue.start, issue.tz) }}</span>
                    <span class="inline-flex px-2 py-1 rounded-full border mono text-[11px] font-bold"
                        :class="check.isNull(issue.end) ? 'bg-slate-50 border-slate-200 text-slate-400' : 'bg-white border-emerald-100 text-green-900'">OUT
                        {{ fmtTime(issue.end, issue.tz) }}</span>
                </div>
                <div class="mt-2 flex flex-wrap gap-1">
                    <span v-if="hasTag(issue, 'late')"
                        class="px-2 py-1 rounded-full bg-red-50 border border-red-200 text-red-700 mono text-[10px] font-bold">Late</span>
                    <span v-if="hasTag(issue, 'early')"
                        class="px-2 py-1 rounded-full bg-amber-50 border border-amber-200 text-amber-700 mono text-[10px] font-bold">Early
                        Leave</span>
                    <span v-if="hasTag(issue, 'puncht')"
                        class="px-2 py-1 rounded-full bg-slate-800 text-white mono text-[10px] font-bold">Absent</span>
                    <span v-if="hasTag(issue, 'overtime')"
                        class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Overtime</span>
                    <span v-if="hasTag(issue, 'present')"
                        class="px-2 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Present</span>
                </div>
                <div class="mt-2 flex items-center justify-between mono text-xs border-t border-emerald-50 pt-2">
                    <span class="text-slate-500">Worked <b class="text-green-900">{{ fmtDur(issue.working_time)
                            }}</b></span>
                    <span class="flex items-center gap-1.5">
                        <span v-if="overtimeSec(issue) > 0"
                            class="px-2 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 mono text-[10px] font-bold">Overtime
                            {{ fmtDur(overtimeSec(issue)) }}</span>
                        <span v-if="issue.late_time > 0"
                            class="px-2 py-1 rounded-full bg-red-50 border border-red-100 text-red-600 mono text-[10px] font-bold">Late
                            {{ fmtDur(issue.late_time) }}</span>
                    </span>
                </div>
            </div>
        </div>
        <div v-else class="md:hidden p-10 text-center">
            <p class="mono text-xs font-bold tracking-widest text-emerald-700">NO DATA</p>
            <p class="text-sm text-slate-500 mt-1">Change the date range or the filters.</p>
        </div>

        <div v-if="contents && contents.length"
            class="sticky bottom-0 bg-white/90 backdrop-blur border-t border-emerald-100 px-4 py-2.5 flex items-center justify-between mono text-[11px]">
            <span class="text-emerald-700/60">{{ contents.length }} rows • filtered from {{ (list.contents || []).length
                }} total</span>
            <span class="hidden sm:inline text-slate-400">Scroll to see all • Export CSV above</span>
        </div>
    </div>
</template>
