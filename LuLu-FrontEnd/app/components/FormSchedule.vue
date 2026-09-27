<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY_LABEL } from '~/utils/labels';
import { useSchedule } from '~/store/schedule';

const
    auth = useAuthStore(),
    schedule = useSchedule(),
    shift = useShiftStore(),
    config = useRuntimeConfig(),
    check = useChecker(),
    emit = defineEmits(['notif', 'done']),
    validator = reactive({ name: '', pattern: '', shift: '' }),
    form_data = reactive({
        name: '',
        pattern: 1,
        shift: [null],
    });

// options: label + value = schedule_type
const patternOptions = [
    { label: 'Daily — one shift for every day', value: 1, desc: 'Flat • DOM 1', badge: 'DAILY' },
    { label: 'Weekly — custom across 7 days', value: 7, desc: '7 days • Sun–Sat', badge: 'WEEKLY' },
    { label: '2 Weeks — custom across 14 days', value: 14, desc: '14 days • 2 weeks', badge: '2 WEEKS' },
    { label: '3 Weeks — custom across 21 days', value: 21, desc: '21 days • 3 weeks', badge: '3 WEEKS' },
    { label: 'Monthly — custom across 28 days (4 weeks)', value: 28, desc: '28 days • 4 weeks', badge: 'MONTHLY' },
]

function patternMeta(v) {
    return patternOptions.find(o => o.value === v) || patternOptions[0]
}

const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
function dayLabel(idx) {
    const d = idx % 7
    const week = Math.floor(idx / 7) + 1
    const base = days[d]
    if (form_data.pattern === 1) return base
    if (form_data.pattern === 7) return base
    return `W${week} — ${base}`
}
function shortDay(idx) { return days[idx % 7].slice(0, 3) }

function ensureShiftLength() {
    const n = form_data.pattern
    if (form_data.shift.length !== n) {
        const cur = form_data.shift.slice()
        form_data.shift = Array.from({ length: n }, (_, i) => cur[i] ?? null)
    }
}
watch(() => form_data.pattern, () => ensureShiftLength(), { immediate: true })

function copyWeek(fromWeek) {
    const n = form_data.pattern
    const weeks = n / 7
    if (weeks <= 1) return
    const srcStart = (fromWeek - 1) * 7
    for (let w = 2; w <= weeks; w++) {
        if (w === fromWeek) continue
        for (let d = 0; d < 7; d++) {
            const srcIdx = srcStart + d
            const dstIdx = (w - 1) * 7 + d
            form_data.shift[dstIdx] = form_data.shift[srcIdx]
        }
    }
}
function fillAllWithFirst() {
    const v = form_data.shift[0]
    for (let i = 1; i < form_data.shift.length; i++) form_data.shift[i] = v
}
function setAllToOff() {
    for (let i = 0; i < form_data.shift.length; i++) form_data.shift[i] = 0
}
function clearShifts() {
    for (let i = 0; i < form_data.shift.length; i++) form_data.shift[i] = null
}
function isHoliday(val) { return val === 0 || val === '0' }

async function ensureShiftsLoaded() {
    if (shift.contents && shift.contents.length) return
    try {
        const r = await $fetch(`${config.public.apiBase}/shift/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) shift.set(r.data)
    } catch (e) { console.error(e) }
}
onMounted(() => { ensureShiftsLoaded(); ensureShiftLength() })

function validate() {
    validator.name = ''; validator.pattern = ''; validator.shift = ''
    if (!form_data.name || form_data.name.trim().length < 3) validator.name = 'Name must be at least 3 characters'
    if (!patternOptions.some(o => o.value === form_data.pattern)) validator.pattern = 'Invalid pattern'
    const n = form_data.pattern
    if (form_data.shift.length !== n) validator.shift = `${n} shifts required`
    else if (form_data.shift.some(v => v === null || v === undefined || v === '')) validator.shift = 'Fill in every day — pick a shift or Day off'
    return !validator.name && !validator.pattern && !validator.shift
}

function callbackNotif(data) { emit('notif', data) }

const submitting = ref(false)
const add_schedule = async () => {
    if (!validate()) return
    submitting.value = true
    validator.shift = ''
    try {
        const body = {
            name: form_data.name.trim(),
            pattern: form_data.pattern,
            shift: form_data.shift.map(v => v === 0 || v === '0' ? 0 : Number(v)),
        }
        const response = await $fetch(`${config.public.apiBase}/schedule/add`, { method: 'POST', body, headers: auth.confHeaders() })
        if (response.code === 200) {
            // reload list
            try {
                const r2 = await $fetch(`${config.public.apiBase}/schedule/list`, { method: 'GET', headers: auth.confHeaders() })
                if (r2.code === 200) schedule.set(r2.data)
            } catch { }
            form_data.name = ''; clearShifts()
            ensureShiftLength()
            callbackNotif({ title: 'Create Schedule', message: response.message || 'Schedule created' })
            emit('done')
        } else {
            if (response.data) {
                validator.name = response.data.name || ''
                validator.shift = response.data.shift || response.data.pattern || ''
            }
            callbackNotif({ title: 'Error', message: response.message || 'Failed to create schedule' })
        }
    } catch (e) {
        const msg = e?.data?.message || e?.message || 'Connection failed'
        // try parse field errors
        if (e?.data?.data) {
            validator.name = e.data.data.name || ''
            validator.shift = e.data.data.shift || ''
        }
        callbackNotif({ title: 'Error', message: msg })
    } finally { submitting.value = false }
};
</script>

<template>
    <div class="text-left">
        <!-- Header -->
        <div
            class="px-5 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-lime-50/40 flex items-center justify-between tech-grid-soft">
            <div class="flex items-center gap-2">
                <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse"></span>
                <h3 class="text-sm font-bold tracking-wide text-green-900 uppercase mono">Create Schedule</h3>
                <span class="chip chip-emerald mono !text-[9px] hidden sm:inline-flex">CUSTOM • DAILY • WEEKLY •
                    MONTHLY</span>
            </div>
            <span class="mono text-[10px] tracking-widest text-emerald-700/50 hidden md:inline">{{
                patternMeta(form_data.pattern).badge }} • {{ patternMeta(form_data.pattern).desc }}</span>
        </div>

        <form @submit.prevent="add_schedule" class="px-5 py-4 space-y-4">
            <!-- Nama + Pattern -->
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
                <div>
                    <label for="schedule_name"
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Schedule
                        Name <span class="text-red-500">*</span></label>
                    <input id="schedule_name" v-model="form_data.name" type="text" required
                        placeholder="e.g. Regular Morning Shift, Warehouse A Roster"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/30 transition">
                    <p v-if="validator.name" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{ validator.name }}
                    </p>
                    <p v-else class="mt-1.5 mono text-[10px] tracking-wide text-emerald-700/40 ml-1">Unique name •
                        becomes the parent id (employees pick this schedule)</p>
                </div>
                <div>
                    <label for="schedule_pattern"
                        class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Schedule
                        Pattern <span class="text-red-500">*</span></label>
                    <select id="schedule_pattern" v-model.number="form_data.pattern"
                        class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 transition">
                        <option v-for="o in patternOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
                    </select>
                    <p v-if="validator.pattern" class="mt-1.5 text-xs font-medium text-red-500 ml-1">{{
                        validator.pattern }}</p>
                    <p v-else class="mt-1.5 mono text-[10px] tracking-wide text-emerald-700/40 ml-1">{{
                        patternMeta(form_data.pattern).desc }} • type={{ form_data.pattern }}</p>
                </div>
            </div>

            <!-- Info bar -->
            <div
                class="hud-frame rounded-xl border border-emerald-100 bg-emerald-50/40 px-3 py-2.5 flex flex-wrap items-center gap-2 text-xs">
                <span class="hud-corner hud-corner-tl hidden sm:block"></span>
                <span class="inline-flex items-center gap-1.5 chip chip-emerald mono !text-[9px]"><span
                        class="w-1.5 h-1.5 rounded-full bg-emerald-500 data-tick"></span> {{ form_data.pattern }}
                    DAYS</span>
                <span class="mono text-emerald-700/60">{{ form_data.pattern === 1 ? '1 shift for all days' :
                    `${form_data.pattern / 7} minggu • ${form_data.pattern} slot` }}</span>
                <span class="ml-auto flex items-center gap-1.5">
                    <button type="button" @click="clearShifts"
                        class="px-2.5 py-1 rounded-full bg-white border border-emerald-200 text-emerald-700 hover:bg-emerald-50 mono text-[10px] tracking-wide">Clear</button>
                    <button type="button" @click="fillAllWithFirst"
                        class="px-2.5 py-1 rounded-full bg-emerald-600 text-white hover:bg-emerald-700 mono text-[10px] tracking-wide">W1→All</button>
                </span>
            </div>

            <!-- FLAT -->
            <div v-if="form_data.pattern === 1" class="hud-frame glass rounded-2xl p-4 border border-emerald-100">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 mb-2">DAILY — 1 SHIFT FOR ALL
                    DAYS</p>
                <label
                    class="block text-xs font-semibold tracking-widest uppercase text-emerald-700/70 mb-1.5 ml-1">Pick
                    Shift <span class="text-red-500">*</span></label>
                <select v-model="form_data.shift[0]"
                    class="w-full px-3.5 h-11 rounded-xl border-2 border-emerald-100 bg-white focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900">
                    <option :value="null" disabled>— Pick a shift —</option>
                    <option :value="0">Day off — no shift</option>
                    <option v-for="d in shift.contents" :key="d.id" :value="d.id">{{ d.name }} — {{ d.start_time != null ?
                        String(Math.floor(d.start_time / 3600)).padStart(2, '0') + ':' + String(Math.floor((d.start_time % 3600) / 60)).padStart(2,'0')
                        : '' }} → {{ d.end_time != null ?
                            String(Math.floor(d.end_time / 3600)).padStart(2, '0') + ':' + String(Math.floor((d.end_time % 3600) / 60)).padStart(2,'0')
                        : '' }}</option>
                </select>
                <p v-if="shift.contents.length === 0"
                    class="mt-2 text-xs text-amber-600 bg-amber-50 border border-amber-200 rounded-lg px-3 py-2">No
                    shifts yet — create them in the Shift tab first</p>
            </div>

            <!-- WEEKLY / BIWEEKLY / MONTHLY GRID -->
            <div v-else class="hud-frame glass rounded-2xl border border-emerald-100 overflow-hidden">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <div
                    class="px-4 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-white flex flex-wrap items-center gap-2">
                    <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700">CUSTOM PER DAY</span>
                    <span class="mono text-[10px] tracking-widest text-emerald-700/50">{{ days.join(' • ') }}</span>
                    <span class="ml-auto mono text-[10px] tracking-wide text-emerald-700/40">Pick a shift per day • 0 =
                        Day off</span>
                </div>

                <!-- column headers Sun..Sat -->
                <div class="hidden sm:grid gap-2 px-3 pt-3"
                    :class="form_data.pattern === 7 ? 'grid-cols-7' : form_data.pattern === 14 ? 'grid-cols-7' : 'grid-cols-7'">
                    <div v-for="d in days" :key="d"
                        class="text-center mono text-[10px] font-bold tracking-widest py-1.5 rounded-lg"
                        :class="d === 'Sunday' ? 'bg-red-500 text-white' : 'bg-emerald-600 text-white'">{{ d.slice(0, 3) }}
                    </div>
                </div>

                <div class="p-3 space-y-3">
                    <template v-for="week in (form_data.pattern / 7)" :key="week">
                        <div class="rounded-xl border border-emerald-100 bg-white/80 p-3">
                            <div class="flex items-center justify-between mb-2">
                                <span class="mono text-xs font-bold tracking-widest text-emerald-700">WEEK {{ week
                                    }}</span>
                                <span class="flex items-center gap-1.5">
                                    <button v-if="week === 1 && form_data.pattern > 7" type="button" @click="copyWeek(1)"
                                        class="mono text-[10px] px-2.5 py-1 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 hover:bg-emerald-100">Copy
                                        W1 → all weeks</button>
                                    <span
                                        class="mono text-[10px] tracking-widest text-emerald-700/40 hidden sm:inline">7
                                        days</span>
                                </span>
                            </div>
                            <!-- mobile: stacked, desktop: 7 cols -->
                            <div class="grid grid-cols-1 sm:grid-cols-7 gap-2">
                                <div v-for="d in 7" :key="d" class="space-y-1">
                                    <label
                                        class="block mono text-[10px] tracking-widest text-emerald-700/60 ml-1 sm:hidden">{{
                                        days[d-1] }} — W{{ week }}</label>
                                    <label
                                        class="hidden sm:block mono text-[9px] tracking-widest text-emerald-700/50 text-center">{{
                                        shortDay(d-1) }}</label>
                                    <select :value="form_data.shift[(week - 1) * 7 + (d - 1)]"
                                        @change="form_data.shift[(week - 1) * 7 + (d - 1)] = $event.target.value === '' ? null : ($event.target.value === '0' ? 0 : Number($event.target.value))"
                                        class="w-full px-2.5 py-2 rounded-xl border-2 bg-white text-xs text-green-900 outline-none transition"
                                        :class="(form_data.shift[(week - 1) * 7 + (d - 1)] === null || form_data.shift[(week - 1) * 7 + (d - 1)] === '') ? 'border-amber-200 bg-amber-50/40 focus:border-amber-400' : isHoliday(form_data.shift[(week - 1) * 7 + (d - 1)]) ? 'border-slate-200 bg-slate-50 focus:border-slate-300' : 'border-emerald-100 focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100'">
                                        <option :value="null" disabled>— Pick —</option>
                                        <option :value="0">{{ OFF_DAY_LABEL }}</option>
                                        <option v-for="s in shift.contents" :key="s.id" :value="s.id">{{ s.name }}
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </template>

                    <p v-if="validator.shift" class="text-xs font-medium text-red-500 ml-1">{{ validator.shift }}</p>
                    <div v-if="shift.contents.length === 0"
                        class="text-xs text-amber-700 bg-amber-50 border border-amber-200 rounded-xl px-3 py-2.5">No
                        shifts yet — create
                        them first, then fill in the schedule day by day.</div>

                    <!-- Preview strip -->
                    <div class="rounded-xl bg-emerald-50/60 border border-emerald-100 px-3 py-2.5">
                        <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 mb-1.5">SUMMARY PREVIEW
                        </p>
                        <div class="flex flex-wrap gap-1.5">
                            <span v-for="(v, idx) in form_data.shift" :key="idx"
                                class="inline-flex items-center gap-1 px-2 py-1 rounded-full border text-[11px] mono"
                                :class="v === null || v === '' ? 'bg-amber-50 border-amber-200 text-amber-700' : isHoliday(v) ? 'bg-slate-50 border-slate-200 text-slate-600' : 'bg-white border-emerald-200 text-emerald-700'">
                                <span class="w-1.5 h-1.5 rounded-full"
                                    :class="v === null ? 'bg-amber-400' : isHoliday(v) ? 'bg-slate-400' : 'bg-emerald-500'"></span>
                                {{ shortDay(idx % 7) }}{{ form_data.pattern > 7 ? ' W' + (Math.floor(idx / 7) + 1) : '' }}: {{
                                    isHoliday(v) ?
                                        OFF_DAY_LABEL : (shift.contents.find(s=>String(s.id)===String(v))?.name ||
                                (v===null?'—':'?')) }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-3 pt-1">
                <button type="submit" :disabled="submitting"
                    class="btn-emerald rounded-xl px-8 h-11 text-sm font-bold mono tracking-widest disabled:opacity-60 disabled:cursor-not-allowed inline-flex items-center gap-2">
                    <svg v-if="submitting" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4">
                        </circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z">
                        </path>
                    </svg>
                    {{ submitting ? 'Menyimpan...' : 'Save Schedule' }}
                </button>
                <button type="button" @click="form_data.name = ''; clearShifts(); ensureShiftLength()"
                    class="rounded-xl px-8 h-11 text-sm font-medium bg-white border-2 border-emerald-100 text-emerald-700 hover:bg-emerald-50 hover:border-emerald-200 transition">Reset</button>
                <span class="mono text-[10px] tracking-widest text-emerald-700/40 ml-auto hidden sm:inline">Shifts from
                    the Shift
                    tab • drag & drop also works from the list on the left</span>
            </div>
        </form>
    </div>
</template>
