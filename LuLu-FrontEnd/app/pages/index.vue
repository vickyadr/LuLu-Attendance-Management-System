<script setup>
import LiveTable from '~/components/LiveTable.vue';
import { useAuthStore } from '~/store/auth';
import { useSettingsStore } from '~/store/settings';
import { useChecker } from '#imports';
import {
  faUserGroup, faDesktop, faCalendarCheck, faChartColumn,
  faPlaneDeparture, faCalendarDays, faUsers, faClock,
  faArrowRight, faArrowUpRightFromSquare, faRotate, faPlus, faLock,
} from '@fortawesome/free-solid-svg-icons';

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
const canShifts = computed(() => auth.hasPageAccess('shift'))

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
  <div class="flex flex-col gap-4 lg:flex-1 lg:min-h-0">

    <!-- ===== Greeting: brand moment, restored ===== -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 shrink-0">

      <section
        class="lg:col-span-5 relative overflow-hidden rounded-2xl bg-gradient-to-br from-emerald-800 via-emerald-700 to-green-800 text-white shadow-tint-md">
        <div class="pointer-events-none absolute inset-0 tech-dots opacity-[0.06]" aria-hidden="true"></div>
        <div class="scanline" style="animation-duration:9s" aria-hidden="true"></div>

        <div class="relative flex flex-col h-full p-4 sm:p-5">
          <div class="flex items-start justify-between gap-3">
            <p class="mono text-[10px] tracking-[0.14em] uppercase text-white/60">{{ todayStr }}</p>
            <span class="mono text-[10px] tracking-[0.08em] text-white/55 inline-flex items-center gap-1.5 shrink-0">
              <span class="w-1.5 h-1.5 rounded-full bg-lime-300"></span>{{ lastSync }}
            </span>
          </div>

          <h2 class="display text-[21px] sm:text-[23px] leading-[1.15] mt-3">
            {{ greeting }},<br>
            <span class="text-white">{{ displayName }}</span>
            <span class="mono text-[10px] font-bold tracking-[0.12em] text-white/55 align-middle ml-1.5">
              <span class="text-lime-300">&#9673;</span> {{ $t('dashboard.liveBadge') }}
            </span>
          </h2>

          <div class="mt-4 flex flex-wrap gap-2">
            <button v-if="canReports" @click="navigateTo('/report')"
              class="inline-flex items-center gap-2 min-h-[40px] px-4 rounded-full bg-white text-emerald-800 text-[12.5px] font-bold hover:bg-emerald-50 active:scale-[0.98] transition shadow-tint-sm">
              {{ $t('dashboard.openReport') }}
              <font-awesome :icon="faArrowRight" class="w-3 h-3" />
            </button>
            <button v-if="canEmployees" @click="navigateTo('/employee')"
              class="inline-flex items-center gap-1.5 min-h-[40px] px-4 rounded-full bg-white/10 border border-white/25 text-white text-[12.5px] font-bold hover:bg-white/15 active:scale-[0.98] transition">
              <font-awesome :icon="faPlus" class="w-3 h-3" />
              {{ $t('dashboard.addEmployee') }}
            </button>
          </div>

          <div class="mt-auto pt-4 flex flex-wrap items-center gap-1.5">
            <span class="chip !bg-white/10 !border-white/20 !text-white/80">
              <span class="w-1.5 h-1.5 rounded-full"
                :class="wsStatus === 'open' ? 'bg-lime-300 data-tick' : 'bg-amber-300'"></span>
              <template v-if="wsStatus === 'open'">{{ $t('dashboard.streaming') }}</template>
              <template v-else-if="wsStatus === 'reconnecting'">{{ $t('dashboard.reconnecting') }}</template>
              <template v-else>{{ $t('live.offline') }}</template>
            </span>
            <button type="button" @click="refreshAll()" :disabled="refreshing"
              class="inline-flex items-center gap-1.5 min-h-[32px] px-2.5 rounded-lg bg-white/10 hover:bg-white/15 text-white mono text-[10px] font-bold uppercase tracking-[0.08em] transition disabled:opacity-60">
              <font-awesome :icon="faRotate" class="w-3 h-3" :class="refreshing ? 'animate-spin' : ''" />
              {{ $t('dashboard.refresh') }}
            </button>
          </div>
        </div>
      </section>

      <!-- the four live counters -->
      <div class="lg:col-span-7 grid grid-cols-2 gap-3 sm:gap-4 auto-rows-fr">

        <button @click="navigateTo('/employee')"
          class="group text-left panel p-4 transition duration-200 hover:-translate-y-0.5 hover:shadow-tint-md hover:border-border-strong">
          <div class="flex items-start justify-between">
            <span class="grid place-content-center w-9 h-9 rounded-lg bg-surface-raised text-primary-deep">
              <font-awesome :icon="faUserGroup" class="w-4 h-4" />
            </span>
            <font-awesome :icon="faArrowUpRightFromSquare"
              class="w-3 h-3 text-text-muted/35 group-hover:text-primary-deep transition" />
          </div>
          <p class="card-label mt-3">{{ $t('dashboard.employees') }}</p>
          <p v-if="!canEmployees" class="mt-1 text-sm font-bold text-text-muted">{{ $t('dashboard.noAccess') }}</p>
          <p v-else-if="stats.karyawan.loading" class="skeleton mt-1.5 h-7 w-14"></p>
          <p v-else class="num text-[24px] text-text mt-1">{{ stats.karyawan.total }}</p>
          <p v-if="stats.karyawan.loading" class="skeleton mt-2 h-3 w-24"></p>
          <p v-else class="mono text-[11px] text-text-muted mt-1 truncate">
            {{ canEmployees ? $t('dashboard.activeInactive', {
              active: stats.karyawan.aktif, inactive: Math.max(0,
                stats.karyawan.total - stats.karyawan.aktif)
            }) : $t('dashboard.noAccessEmployee') }}
          </p>
        </button>

        <button @click="navigateTo('/controller')"
          class="group text-left panel p-4 transition duration-200 hover:-translate-y-0.5 hover:shadow-tint-md hover:border-border-strong">
          <div class="flex items-start justify-between">
            <span class="grid place-content-center w-9 h-9 rounded-lg bg-surface-raised text-primary-deep">
              <font-awesome :icon="faDesktop" class="w-4 h-4" />
            </span>
            <font-awesome :icon="faArrowUpRightFromSquare"
              class="w-3 h-3 text-text-muted/35 group-hover:text-primary-deep transition" />
          </div>
          <p class="card-label mt-3">{{ $t('dashboard.devices') }}</p>
          <p v-if="!canDevices" class="mt-1 text-sm font-bold text-text-muted">{{ $t('dashboard.noAccess') }}</p>
          <p v-else-if="stats.device.loading" class="skeleton mt-1.5 h-7 w-14"></p>
          <p v-else class="num text-[24px] text-text mt-1">{{ stats.device.total }}</p>
          <p v-if="stats.device.loading" class="skeleton mt-2 h-3 w-24"></p>
          <p v-else class="mono text-[11px] mt-1 inline-flex items-center gap-1.5 truncate"
            :class="canDevices && stats.device.online === 0 ? 'text-warn' : 'text-text-muted'">
            <span v-if="canDevices" class="w-1.5 h-1.5 rounded-full shrink-0"
              :class="stats.device.online > 0 ? 'bg-primary' : 'bg-amber-500'"></span>
            {{ canDevices ? $t('dashboard.onlineOffline', {
              online: stats.device.online, offline: Math.max(0,
                stats.device.total - stats.device.online)
            }) : $t('dashboard.noAccessDevice') }}
          </p>
        </button>

        <button @click="navigateTo('/report')"
          class="group text-left panel p-4 transition duration-200 hover:-translate-y-0.5 hover:shadow-tint-md hover:border-border-strong">
          <div class="flex items-start justify-between">
            <span class="grid place-content-center w-9 h-9 rounded-lg bg-surface-raised text-primary-deep">
              <font-awesome :icon="faCalendarCheck" class="w-4 h-4" />
            </span>
            <font-awesome :icon="faArrowUpRightFromSquare"
              class="w-3 h-3 text-text-muted/35 group-hover:text-primary-deep transition" />
          </div>
          <p class="card-label mt-3">{{ $t('dashboard.today') }}</p>
          <p v-if="!canReports" class="mt-1 text-sm font-bold text-text-muted">{{ $t('dashboard.noAccess') }}</p>
          <p v-else-if="stats.today.loading" class="skeleton mt-1.5 h-7 w-16"></p>
          <p v-else class="num text-[24px] text-text mt-1">{{ stats.today.total }}
            <span class="mono text-[11px] text-text-muted">{{ $t('dashboard.records') }}</span>
          </p>
          <p v-if="stats.today.loading" class="skeleton mt-2 h-4 w-28"></p>
          <p v-else-if="!canReports" class="mono text-[11px] text-text-muted mt-1">{{ $t('dashboard.noAccessReport') }}
          </p>
          <p v-else class="mt-1.5 flex flex-wrap gap-1">
            <span class="chip chip-ok">{{ $t('dashboard.presentCount', { count: stats.today.hadir }) }}</span>
            <span class="chip" :class="stats.today.terlambat > 0 ? 'chip-danger' : 'chip-quiet'">{{
              $t('dashboard.lateCount',
                { count: stats.today.terlambat }) }}</span>
          </p>
        </button>

        <button @click="navigateTo('/report')"
          class="group text-left panel p-4 transition duration-200 hover:-translate-y-0.5 hover:shadow-tint-md hover:border-border-strong">
          <div class="flex items-start justify-between">
            <span class="grid place-content-center w-9 h-9 rounded-lg bg-surface-raised text-primary-deep">
              <font-awesome :icon="faChartColumn" class="w-4 h-4" />
            </span>
            <font-awesome :icon="faArrowUpRightFromSquare"
              class="w-3 h-3 text-text-muted/35 group-hover:text-primary-deep transition" />
          </div>
          <p class="card-label mt-3">{{ $t('live.title') }}</p>
          <p v-if="!canReports" class="mt-1 text-sm font-bold text-text-muted">{{ $t('dashboard.noAccess') }}</p>
          <p v-else-if="stats.live.loading" class="skeleton mt-1.5 h-7 w-14"></p>
          <p v-else class="num text-[24px] text-text mt-1">{{ stats.live.count }}
            <span class="mono text-[11px] text-text-muted">{{ $t('dashboard.transactions') }}</span>
          </p>
          <p class="mono text-[11px] text-text-muted mt-1 truncate">{{ canReports ? $t('dashboard.latestSynced', {
            count: 100,
            time: lastSync
          }) : $t('dashboard.noAccessReport') }}</p>
        </button>
      </div>
    </div>

    <!-- ===== Secondary counts: one strip, not four more cards ===== -->
    <div class="panel shrink-0 grid grid-cols-2 lg:grid-cols-4 divide-x divide-border overflow-hidden">
      <button v-if="canLeave" @click="navigateTo('/leave')"
        class="group text-left px-4 py-3 transition hover:bg-surface-raised/60 min-h-[64px]">
        <p class="card-label">{{ $t('nav.leave') }}</p>
        <p v-if="stats.leave.loading" class="skeleton mt-1.5 h-5 w-12"></p>
        <p v-else class="num text-[18px] text-text mt-0.5">{{ stats.leave.pending }}
          <span class="mono text-[10.5px] font-bold text-text-muted">{{ $t('leave.stPending') }}</span>
        </p>
      </button>

      <button v-if="canSettings" @click="navigateTo('/settings')"
        class="group text-left px-4 py-3 transition hover:bg-surface-raised/60 min-h-[64px] min-w-0">
        <p class="card-label">{{ $t('holiday.title') }}</p>
        <p v-if="stats.holiday.loading" class="skeleton mt-1.5 h-5 w-20"></p>
        <p v-else class="text-[12.5px] font-semibold text-text mt-0.5 truncate" :title="stats.holiday.nextName || ''">
          {{ stats.holiday.nextName || $t('common.none') }}
        </p>
        <p v-if="!stats.holiday.loading && stats.holiday.nextDate" class="mono text-[10.5px] text-text-muted">{{
          stats.holiday.nextDate }}</p>
      </button>

      <button v-if="canUsers" @click="navigateTo('/users')"
        class="group text-left px-4 py-3 transition hover:bg-surface-raised/60 min-h-[64px]">
        <p class="card-label">{{ $t('nav.users') }}</p>
        <p v-if="stats.users.loading" class="skeleton mt-1.5 h-5 w-10"></p>
        <p v-else class="num text-[18px] text-text mt-0.5">{{ stats.users.total }}
          <span class="mono text-[10.5px] font-bold text-text-muted">{{ $t('user.stAccounts') }}</span>
        </p>
      </button>

      <button v-if="canShifts" @click="navigateTo('/shift')"
        class="group text-left px-4 py-3 transition hover:bg-surface-raised/60 min-h-[64px] min-w-0">
        <p class="card-label">{{ $t('dashboard.shiftsSchedules') }}</p>
        <p class="text-[12.5px] font-semibold text-text mt-0.5 truncate">{{ $t('dashboard.shiftsSchedulesSub') }}</p>
      </button>
    </div>

    <!-- ===== Live feed ===== -->
    <LiveTable v-if="canReports" :ws-status="wsStatus" :refresh="liveRefresh" />
  </div>
</template>
