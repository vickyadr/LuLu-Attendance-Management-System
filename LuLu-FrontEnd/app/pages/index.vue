<script setup>
import LiveTable from '~/components/LiveTable.vue';
import { useAuthStore } from '~/store/auth';
import { useSettingsStore } from '~/store/settings';
import { useChecker } from '#imports';

useHead({ title: "LuLu - Dashboard" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

const auth = useAuthStore()
const config = useRuntimeConfig()
const check = useChecker()
const { t, locale } = useI18n()

const greeting = computed(() => {
    const h = new Date().getHours()
    if (h < 11) return t('dashboard.greetingMorning')
    if (h < 15) return t('dashboard.greetingAfternoon')
    if (h < 18) return t('dashboard.greetingEvening')
    return t('dashboard.greetingNight')
})
const todayStr = computed(() => {
    try { return new Intl.DateTimeFormat(locale.value === 'id' ? 'id-ID' : 'en-GB', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' }).format(new Date()) } catch { return '' }
})
// Store exposes displayName/initials as plain unwrapped computeds; reading
// auth.user.fname.value here returned undefined and silently showed the fallback.
const displayName = computed(() => auth.displayName)

// Each card reads a page that is gated under its own page_key (employee/list ->
// "employee", device/list -> "controller", report/today -> "report",
// transaction/live -> "report"). A user without that permission gets a 403, and
// because the fetches swallow non-200 replies the card would render a convincing
// but false "0". Track access per card and show a locked state instead.
const canEmployees = computed(() => auth.hasPageAccess('employee'))
const canDevices = computed(() => auth.hasPageAccess('controller'))
const canReports = computed(() => auth.hasPageAccess('report'))
const canLeave = computed(() => auth.hasPageAccess('leave'))
const canSettings = computed(() => auth.hasPageAccess('settings'))
const canUsers = computed(() => auth.hasPageAccess('users'))

const stats = reactive({
    karyawan: { total: 0, aktif: 0, loading: true },
    device: { total: 0, online: 0, loading: true },
    today: { total: 0, hadir: 0, terlambat: 0, puncht: 0, early: 0, overtime: 0, loading: true },
    live: { count: 0, loading: true },
    leave: { pending: 0, loading: true },
    holiday: { nextDate: '', nextName: '', count: 0, loading: true },
    users: { total: 0, loading: true },
})
const lastSync = ref('-')
const refreshing = ref(false)

const settings = useSettingsStore()
const overtimeTol = computed(() => Number(settings.overtimeTolSec) || 0)

function statusTagsOf(it) {
    const tags = []
    const isAbsen = check.isNull(it.start) && check.isNull(it.end)
    if (it.late_time > 0) tags.push('late')
    // Early/overtime follows the same rule as Report: compare the actual OUT
    // against the shift end, and take the overtime value from the backend (which
    // already applies `overtime_tolerance_sec` from the settings table).
    if (!isAbsen && !check.isNull(it.end)) {
        const tz = Number(it.tz) || 0
        const endRef = (Number(it.date) || 0) - tz * 3600 + (it.shift_end ?? 0)
        if (Number(it.end) < endRef - overtimeTol.value) tags.push('early')
    }
    if (isAbsen) tags.push('puncht')
    if (Number(it.overtime) > 0) tags.push('overtime')
    if (tags.length === 0 && !isAbsen) tags.push('present')
    return tags
}

async function fetchKaryawan() {
    if (!canEmployees.value) return
    stats.karyawan.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            const arr = r.data || []
            stats.karyawan.total = arr.length
            stats.karyawan.aktif = arr.filter(x => String(x.status) === '0').length
        }
    } catch (e) { } finally { stats.karyawan.loading = false }
}
async function fetchDevices(quiet) {
    if (!canDevices.value) return
    // `quiet` matches fetchToday/fetchLive: on a live event the card is
    // already on screen, so only the initial load shows the skeleton.
    if (!quiet) stats.device.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/device/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            const arr = r.data || []
            stats.device.total = arr.length
            stats.device.online = arr.filter(d => d.status === 1 || String(d.status) === '1').length
        }
    } catch (e) { } finally { stats.device.loading = false }
}
async function fetchToday(quiet) {
    if (!canReports.value) return
    // `quiet` skips the skeleton: on a live event the numbers are already on
    // screen, and swapping them for a placeholder on every scan reads as flicker.
    if (!quiet) stats.today.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/report/today`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            const arr = r.data || []
            stats.today.total = arr.length
            let hadir = 0, terlambat = 0, puncht = 0, early = 0, overtime = 0
            for (const it of arr) {
                const tags = statusTagsOf(it)
                if (tags.includes('present')) hadir++
                if (tags.includes('late')) terlambat++
                if (tags.includes('puncht')) puncht++
                if (tags.includes('early')) early++
                if (tags.includes('overtime')) overtime++
            }
            stats.today.hadir = hadir
            stats.today.terlambat = terlambat
            stats.today.puncht = puncht
            stats.today.early = early
            stats.today.overtime = overtime
        }
    } catch (e) { } finally { stats.today.loading = false }
}
async function fetchLive(quiet) {
    if (!canReports.value) return
    if (!quiet) stats.live.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/transaction/live`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            stats.live.count = (r.data || []).length
            lastSync.value = new Date().toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })
        }
    } catch (e) { } finally { stats.live.loading = false }
}

async function fetchLeave() {
    if (!canLeave.value) return
    stats.leave.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/leave/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            stats.leave.pending = (r.data || []).filter(x => x.leave_status === 'pending').length
        }
    } catch (e) { } finally { stats.leave.loading = false }
}

// The next holiday is the first stored date still in the future, and it is the
// one thing a person checks before planning a shift, so it earns a dashboard
// slot. `holiday/list` is gated under "settings", same as the panel itself.
async function fetchHoliday() {
    if (!canSettings.value) return
    stats.holiday.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/holiday/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200 && Array.isArray(r.data)) {
            const arr = r.data
            stats.holiday.count = arr.length
            const today = new Date().toISOString().slice(0, 10)
            const next = arr.filter(h => String(h.holiday_date) > today).sort((a, b) => String(a.holiday_date).localeCompare(String(b.holiday_date)))[0]
            stats.holiday.nextDate = next ? String(next.holiday_date) : ''
            stats.holiday.nextName = next ? String(next.holiday_name || '') : ''
        }
    } catch (e) { } finally { stats.holiday.loading = false }
}

async function refreshAll() {
    refreshing.value = true
    await Promise.allSettled([fetchKaryawan(), fetchDevices(), fetchToday(), fetchLive(), fetchLeave(), fetchHoliday(), fetchUsers()])
    refreshing.value = false
}

// Users is the one count no other card shows: it answers "how many logins
// exist". `user/list` is gated under "users", so non-admins never fetch.
async function fetchUsers() {
    if (!canUsers.value) return
    stats.users.loading = true
    try {
        const r = await apiFetch(`${config.public.apiBase}/user/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200 && Array.isArray(r.data)) {
            stats.users.total = r.data.length
        }
    } catch (e) { } finally { stats.users.loading = false }
}

// Cards whose page the user cannot open must not sit on their skeleton
// forever (the fetch returns early above): mark them settled immediately.
function settleLockedCards() {
    if (!canEmployees.value) stats.karyawan.loading = false
    if (!canDevices.value) stats.device.loading = false
    if (!canReports.value) { stats.today.loading = false; stats.live.loading = false }
    if (!canLeave.value) stats.leave.loading = false
    if (!canSettings.value) stats.holiday.loading = false
    if (!canUsers.value) stats.users.loading = false
}

// One socket for the whole dashboard. The backend broadcasts two events:
// `enroll` when a device posts a new record (the Today card, the live count and
// the live table all move), and `device` when the sweeper flips a controller
// between online and offline (only the Devices card moves). Each event means
// "something on screen may have changed", so the affected card re-reads here.
// Karyawan is never refreshed live: nothing in the feed can change the roster.
const liveRefresh = ref(0)
let syncTimer = null
const { status: wsStatus } = useLiveFeed((data) => {
    const ev = data?.event
    if (ev !== 'enroll' && ev !== 'device') return
    if (ev === 'enroll') liveRefresh.value++
    // Coalesce: a device posting several batches in a row should cost one
    // re-read, not one per batch.
    if (syncTimer) return
    syncTimer = setTimeout(() => {
        syncTimer = null
        fetchToday(true)
        fetchLive(true)
        fetchDevices(true)
        // lastSync still moves for a card the user may not read; harmless.
        lastSync.value = new Date().toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' })
    }, 700)
})

onUnmounted(() => { if (syncTimer) clearTimeout(syncTimer) })

onMounted(() => {
    settings.fetchSettings()
    refreshAll()
    settleLockedCards()
})
</script>

<template>
    <div class="flex flex-col gap-3 lg:flex-1 lg:min-h-0">
        <!-- Greeting / Hero - compact so viewport fits without scrolling on 900px -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-3 shrink-0">
            <!-- Welcome - 5 cols -->
            <div
                class="lg:col-span-5 hud-frame rounded-2xl p-2.5 sm:p-3 bg-gradient-to-br from-emerald-600 via-emerald-600 to-green-700 text-white relative overflow-hidden card-lift neon-ring">
                <div class="pointer-events-none absolute inset-0 tech-dots opacity-10"></div>
                <div class="scanline" style="animation-duration:4s"></div>
                <div class="absolute -right-10 -top-10 w-44 h-44 bg-white/10 rounded-full blur-2xl"></div>
                <div class="absolute -left-8 -bottom-8 w-36 h-36 bg-lime-300/20 rounded-full blur-xl"></div>
                <div class="relative">
                    <div class="flex items-center gap-2 flex-wrap">


                        <span
                            class="ml-auto mono text-[10px] tracking-widest text-white/60 hidden sm:inline-flex items-center gap-1.5"><span
                                class="w-1.5 h-1.5 rounded-full bg-lime-300 animate-pulse"></span> {{ lastSync }} • {{
                            $t('dashboard.sync') }}</span>
                    </div>
                    <p class="mono text-white/70 text-[11px] tracking-[0.14em] uppercase mt-1.5">{{ todayStr }}</p>
                    <h2 class="display text-xl font-extrabold tracking-tight mt-0.5 leading-none">{{ greeting }},
                        <br><span class="text-white">{{ displayName }}</span> <span
                            class="mono text-[11px] font-bold tracking-widest text-white/60 align-middle ml-1">◉ {{
                                $t('dashboard.liveBadge') }}</span></h2>
                    <div class="mt-2.5 flex flex-wrap gap-2">
                        <button v-if="canReports" @click="navigateTo('/report')"
                            class="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-white text-emerald-700 mono text-xs font-extrabold hover:bg-emerald-50 transition shadow-sm">{{
                                $t('dashboard.openReport') }} <span
                                class="w-5 h-5 rounded-full bg-emerald-600 text-white flex items-center justify-center text-[10px]">→</span></button>
                        <button v-if="canEmployees" @click="navigateTo('/employee')"
                            class="inline-flex items-center gap-1.5 px-4 py-2 rounded-full bg-white/15 backdrop-blur text-white mono text-xs font-bold border border-white/20 hover:bg-white/20 transition">+
                            {{ $t('dashboard.addEmployee') }}</button>
                    </div>
                    <div class="mt-2.5 flex flex-wrap gap-1.5 mono text-[10px]">
                        <span
                            class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/10 border border-white/20 text-white/80"><span
                                class="w-1.5 h-1.5 rounded-full"
                                :class="wsStatus === 'open' ? 'bg-lime-300 animate-pulse' : 'bg-amber-300'"></span> <template
                                v-if="wsStatus === 'open'">{{ $t('dashboard.streaming') }}</template><template
                                v-else-if="wsStatus === 'reconnecting'">{{ $t('dashboard.reconnecting')
                                }}</template><template v-else>{{ $t('live.offline') }}</template></span>
                        <button @click="refreshAll()" :disabled="refreshing"
                            class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/10 hover:bg-white/15 text-white mono text-[10px] font-bold disabled:opacity-60">
                            <svg v-if="refreshing" class="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor"
                                    stroke-width="4" />
                                <path class="opacity-75" fill="currentColor"
                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                            </svg>
                            <span v-else class="w-3 h-3 flex items-center justify-center">↻</span> {{
                                $t('dashboard.refresh') }}
                        </button>
                    </div>
                </div>
            </div>

            <!-- Stats - 7 cols : 2x2 -->
            <div class="lg:col-span-7 grid grid-cols-2 gap-2 sm:gap-3 content-start">
                <!-- Employees -->
                <button @click="navigateTo('/employee')"
                    class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div
                        class="w-8 h-8 rounded-lg bg-emerald-100 flex items-center justify-center text-emerald-600 group-hover:bg-emerald-600 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                            stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round"
                                d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
                        </svg>
                    </div>
                    <p
                        class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2 flex items-center gap-1.5">
                        {{
                            $t('dashboard.employees') }} <span v-if="canEmployees"
                            class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span></p>
                    <p v-if="!canEmployees"
                        class="mt-1 text-base font-bold text-emerald-700/50 leading-tight tracking-tight">{{
                            $t('dashboard.noAccess') }}</p>
                    <p v-else-if="stats.karyawan.loading" class="mt-1 h-7 w-12 rounded bg-emerald-100 animate-pulse">
                    </p>
                    <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{
                        stats.karyawan.total }}</p>
                    <p v-if="canEmployees && !stats.karyawan.loading" class="mono text-[11px] text-emerald-700/60 mt-1">
                        {{
                            $t('dashboard.activeInactive', {
                                active: stats.karyawan.aktif, inactive: Math.max(0,
                                    stats.karyawan.total -
                        stats.karyawan.aktif) }) }}</p>
                    <p v-else-if="canEmployees" class="mt-1 h-3 w-20 rounded bg-emerald-50 animate-pulse"></p>
                    <p v-else class="mono text-[11px] text-emerald-700/40 mt-1">{{ $t('dashboard.noAccessEmployee') }}
                    </p>
                    <span
                        class="absolute top-3 right-3 w-6 h-6 rounded-full bg-emerald-50 border border-emerald-100 text-emerald-600 flex items-center justify-center mono text-[10px] group-hover:bg-emerald-600 group-hover:text-white transition">↗</span>
                </button>

                <!-- Device -->
                <button @click="navigateTo('/controller')"
                    class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div
                        class="w-8 h-8 rounded-lg bg-sky-50 border border-sky-100 flex items-center justify-center text-sky-600 group-hover:bg-sky-600 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                            stroke="currentColor" stroke-width="1.8">
                            <rect x="3" y="4" width="18" height="14" rx="2" />
                            <path d="M8 20h8" />
                            <path d="M12 16v4" />
                        </svg>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{
                        $t('dashboard.devices') }}
                    </p>
                    <p v-if="!canDevices"
                        class="mt-1 text-base font-bold text-emerald-700/50 leading-tight tracking-tight">{{
                            $t('dashboard.noAccess') }}</p>
                    <p v-else-if="stats.device.loading" class="mt-1 h-7 w-12 rounded bg-sky-100 animate-pulse"></p>
                    <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{ stats.device.total
                        }}</p>
                    <p v-if="canDevices && !stats.device.loading" class="mono text-[11px] mt-1"
                        :class="stats.device.online > 0 ? 'text-emerald-600' : 'text-amber-600'"><span
                            class="inline-flex items-center gap-1"><span class="w-1.5 h-1.5 rounded-full"
                                :class="stats.device.online > 0 ? 'bg-emerald-500' : 'bg-amber-500'"></span> {{
                                    $t('dashboard.onlineOffline', {
                                        online: stats.device.online, offline: Math.max(0,
                            stats.device.total -
                            stats.device.online) }) }}</span></p>
                    <p v-else-if="canDevices" class="mt-1 h-3 w-20 rounded bg-sky-50 animate-pulse"></p>
                    <p v-else class="mono text-[11px] text-emerald-700/40 mt-1">{{ $t('dashboard.noAccessDevice') }}</p>
                    <span
                        class="absolute top-3 right-3 w-6 h-6 rounded-full bg-sky-50 border border-sky-100 text-sky-600 flex items-center justify-center mono text-[10px] group-hover:bg-sky-600 group-hover:text-white transition">↗</span>
                </button>

                <!-- Today -->
                <button @click="navigateTo('/report')"
                    class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div
                        class="w-8 h-8 rounded-lg bg-amber-50 border border-amber-100 flex items-center justify-center text-amber-600 group-hover:bg-amber-500 group-hover:text-white transition">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                            stroke="currentColor" stroke-width="1.8">
                            <rect x="3" y="4" width="18" height="16" rx="2" />
                            <path d="M16 2v4M8 2v4M3 10h18" />
                            <path d="M9 16l2 2 4-4" />
                        </svg>
                    </div>
                    <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{
                        $t('dashboard.today') }}</p>
                    <p v-if="!canReports"
                        class="mt-1 text-base font-bold text-emerald-700/50 leading-tight tracking-tight">{{
                            $t('dashboard.noAccess') }}</p>
                    <p v-else-if="stats.today.loading" class="mt-1 h-7 w-16 rounded bg-amber-100 animate-pulse"></p>
                    <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{ stats.today.total
                        }} <span class="mono text-xs font-bold text-emerald-700/40">{{ $t('dashboard.records') }}</span>
                    </p>
                    <p v-if="!canReports" class="mono text-[11px] text-emerald-700/40 mt-1">{{
                        $t('dashboard.noAccessReport') }}</p>
                    <p v-else-if="!stats.today.loading"
                        class="mono text-[11px] text-slate-600 mt-1 flex flex-wrap gap-1">
                        <span
                            class="inline-flex px-1.5 py-0.5 rounded-full bg-emerald-50 border border-emerald-200 text-emerald-700 font-bold">{{
                                $t('dashboard.presentCount', { count: stats.today.hadir }) }}</span>
                        <span v-if="stats.today.terlambat > 0"
                            class="inline-flex px-1.5 py-0.5 rounded-full bg-red-50 border border-red-200 text-red-600 font-bold">{{
                                $t('dashboard.lateCount', { count: stats.today.terlambat }) }}</span>
                        <span v-else
                            class="inline-flex px-1.5 py-0.5 rounded-full bg-white border border-emerald-100 text-emerald-700/60">{{
                                $t('dashboard.lateCount', { count: 0 }) }}</span>
                    </p>
                    <p v-else-if="canReports" class="mt-1 h-3 w-24 rounded bg-amber-50 animate-pulse"></p>
                    <span
                        class="absolute top-3 right-3 w-6 h-6 rounded-full bg-amber-50 border border-amber-100 text-amber-600 flex items-center justify-center mono text-[10px] group-hover:bg-amber-500 group-hover:text-white transition">↗</span>
                </button>

                <!-- Live -->
                <div
                    class="hud-frame glass rounded-2xl p-2.5 sm:p-3 border border-emerald-100 relative overflow-hidden flex flex-col">
                    <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                    <div
                        class="w-8 h-8 rounded-lg bg-lime-100 border border-lime-200 flex items-center justify-center text-lime-700 relative">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                            stroke="currentColor" stroke-width="1.8">
                            <path
                                d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
                        </svg>
                        <span
                            class="absolute w-2 h-2 rounded-full bg-lime-500 animate-pulse -translate-y-1 translate-x-3"></span>
                    </div>
                    <p
                        class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2 flex items-center gap-1.5">
                        <span v-if="canReports" class="w-1.5 h-1.5 rounded-full bg-lime-500 animate-pulse"></span> {{
                        $t('live.title')
                        }}</p>
                    <p v-if="!canReports"
                        class="mt-1 text-base font-bold text-emerald-700/50 leading-tight tracking-tight">{{
                            $t('dashboard.noAccess') }}</p>
                    <p v-else-if="stats.live.loading" class="mt-1 h-7 w-12 rounded bg-lime-100 animate-pulse"></p>
                    <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{ stats.live.count }}
                        <span class="mono text-xs font-bold text-emerald-700/40">{{ $t('dashboard.transactions')
                            }}</span></p>
                    <p class="mono text-[11px] text-emerald-700/50 mt-1">{{ canReports ? $t('dashboard.latestSynced', {
                        count: 100,
                        time: lastSync }) : $t('dashboard.noAccessReport') }}</p>
                </div>
            </div>
        </div>

        <!-- Data cards the four stat cards do not already cover: pending leave
             needs a decision, the next holiday changes shift planning, and
             shifts has no stat card of its own. -->
        <div class="grid grid-cols-2 lg:grid-cols-4 gap-3 shrink-0">
            <!-- Leave: pending count, the one number that means "act on me" -->
            <button v-if="canLeave" @click="navigateTo('/leave')"
                class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <div
                    class="w-8 h-8 rounded-lg bg-rose-50 border border-rose-100 flex items-center justify-center text-rose-600 group-hover:bg-rose-500 group-hover:text-white transition">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                        stroke="currentColor" stroke-width="1.8">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25" />
                    </svg>
                </div>
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{ $t('nav.leave') }}
                </p>
                <p v-if="stats.leave.loading" class="mt-1 h-7 w-16 rounded bg-rose-100 animate-pulse"></p>
                <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{ stats.leave.pending }}
                    <span class="mono text-xs font-bold text-emerald-700/40">{{ $t('leave.stPending') }}</span></p>
            </button>

            <!-- Holiday: the next stored non-working day, i.e. the next day the roster must cover -->
            <button v-if="canSettings" @click="navigateTo('/settings')"
                class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <div
                    class="w-8 h-8 rounded-lg bg-violet-50 border border-violet-100 flex items-center justify-center text-violet-600 group-hover:bg-violet-500 group-hover:text-white transition">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                        stroke="currentColor" stroke-width="1.8">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M6.75 3v2.25M17.25 3v2.25M3 18.75V7.5a2.25 2.25 0 012.25-2.25h13.5A2.25 2.25 0 0121 7.5v11.25m-18 0A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75m-18 0v-7.5A2.25 2.25 0 015.25 9h13.5A2.25 2.25 0 0121 11.25v7.5" />
                    </svg>
                </div>
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{ $t('holiday.title')
                    }}</p>
                <p v-if="stats.holiday.loading" class="mt-1 h-7 w-24 rounded bg-violet-100 animate-pulse"></p>
                <template v-else-if="stats.holiday.nextDate">
                    <p class="text-lg font-extrabold text-green-900 mt-1 tracking-tight leading-tight truncate">{{
                        stats.holiday.nextName }}</p>
                    <p class="mono text-[11px] text-violet-700/70 mt-1 font-bold">{{ stats.holiday.nextDate }}</p>
                </template>
                <p v-else class="mono text-[11px] text-emerald-700/50 mt-1">{{ $t('common.none') }}</p>
            </button>

            <!-- Users: how many accounts can sign in at all; admin-only, so it
                 is the one count an ordinary user never sees elsewhere -->
            <button v-if="canUsers" @click="navigateTo('/users')"
                class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <div
                    class="w-8 h-8 rounded-lg bg-indigo-50 border border-indigo-100 flex items-center justify-center text-indigo-600 group-hover:bg-indigo-600 group-hover:text-white transition">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                        stroke="currentColor" stroke-width="1.8">
                        <path stroke-linecap="round" stroke-linejoin="round"
                            d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z" />
                    </svg>
                </div>
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{ $t('nav.users') }}
                </p>
                <p v-if="stats.users.loading" class="mt-1 h-7 w-12 rounded bg-indigo-100 animate-pulse"></p>
                <p v-else class="text-xl font-extrabold text-green-900 mt-0.5 tracking-tight">{{ stats.users.total }}
                    <span class="mono text-xs font-bold text-emerald-700/40">{{ $t('user.stAccounts') }}</span></p>
            </button>

            <!-- Shifts: no stat card above, so this is its only entry point -->
            <button v-if="auth.hasPageAccess('shift')" @click="navigateTo('/shift')"
                class="text-left hud-frame glass rounded-2xl p-2.5 sm:p-3 card-lift border border-emerald-100 relative overflow-hidden hover:border-emerald-200 group">
                <span class="hud-corner hud-corner-tr hidden sm:block"></span>
                <div
                    class="w-8 h-8 rounded-lg bg-sky-50 border border-sky-100 flex items-center justify-center text-sky-600 group-hover:bg-sky-600 group-hover:text-white transition">
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24"
                        stroke="currentColor" stroke-width="1.8">
                        <circle cx="12" cy="12" r="9" />
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 7v5l3 2" />
                    </svg>
                </div>
                <p class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700/60 mt-2">{{
                    $t('dashboard.shiftsSchedules') }}</p>
                <p class="text-sm font-bold text-green-900 mt-1 leading-tight tracking-tight">{{
                    $t('dashboard.shiftsSchedulesSub') }}</p>
            </button>
        </div>

        <!-- Live Transactions - reads transaction/live, gated under "report" -->
        <LiveTable v-if="canReports" :ws-status="wsStatus" :refresh="liveRefresh" />
    </div>
</template>
