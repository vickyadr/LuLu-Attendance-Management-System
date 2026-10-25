<script setup>
import { useReportStore } from '~/store/report';
import { useAuthStore } from '~/store/auth';
import { useSettingsStore } from '~/store/settings';
import { useChecker, useFormater } from '#imports';
import { faDownload, faMagnifyingGlass, faXmark, faRotate } from '@fortawesome/free-solid-svg-icons';

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
  const s = new Set((list.contents || []).map(x => x.department).filter(Boolean))
  return [...s].sort()
})
const filteredPreview = computed(() => {
  let arr = list.contents || []
  const q = (filter.value || '').trim().toLowerCase()
  if (q) arr = arr.filter(item => (`${item.first_name || ''} ${item.last_name || ''} ${item.department || ''}`).toLowerCase().includes(q))
  if (deptFilter.value) arr = arr.filter(item => (item.department || '') === deptFilter.value)
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
  function rowTags(it) {
    const t = statusTagsOf(it)
    const map = { present: 'Present', late: 'Late', early: 'Early Leave', puncht: 'Absent', overtime: 'Overtime', holiday: 'Holiday' }
    return t.map(x => map[x]).join(' | ')
  }
  const lines = [header.map(csvCell).join(',')]
  for (const it of rows) {
    lines.push([
      `${it.first_name || ''} ${it.last_name || ''}`.trim(),
      it.department || '',
      format.stamp_to_naive_date(it.date),
      it.shift_name || '',
      it.schedule_name || '',
      check.isNull(it.start) ? '-' : tzClock(it.start, it.tz),
      check.isNull(it.end) ? '-' : tzClock(it.end, it.tz),
      rowTags(it),
      it.holiday_name || '',
      format.sec_to_naive(it.working_time || 0),
      it.late_time > 0 ? format.sec_to_naive(it.late_time) : '-'
    ].map(csvCell).join(','))
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
  <div class="flex flex-col gap-4">

    <!-- ===== page head: no card, just hierarchy ===== -->
    <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
      <div class="min-w-0">
        <h1 class="display text-[20px] text-text">{{ $t('report.title') }}</h1>
        <div class="mt-1.5 flex flex-wrap items-center gap-2">
          <span class="chip chip-emerald">{{ rangeLabel }}</span>
          <span class="mono text-[10.5px] text-text-muted inline-flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full" :class="loading ? 'bg-amber-500 data-tick' : 'bg-primary'"></span>
            {{ allCount }} data · {{ filteredCount }} shown
          </span>
        </div>
      </div>

      <div class="sm:ml-auto flex flex-wrap items-center gap-2">
        <button @click="exportCsv()" :disabled="!filteredCount" class="btn btn-secondary btn-sm">
          <font-awesome :icon="faDownload" class="w-3.5 h-3.5" />
          {{ $t('report.exportCsv') }}
        </button>
        <button @click="preset('today'); clearFilters()" class="btn btn-primary btn-sm">
          <font-awesome :icon="faRotate" class="w-3.5 h-3.5" />
          {{ $t('report.loadToday') }}
        </button>
      </div>
    </div>

    <!-- ===== controls ===== -->
    <div class="panel p-3 sm:p-4 space-y-3">
      <div class="flex flex-col lg:flex-row lg:items-center gap-3">
        <div class="flex items-center gap-2">
          <SimpleDatePicker :use-utc="true" :init-start-date="format.stamp_to_naive_date()"
            :init-end-date="format.stamp_to_naive_date()" v-on:date-range-selected="getDataReport" />
          <span v-if="loading" class="inline-flex items-center gap-1.5 mono text-[11px] text-text-muted">
            <font-awesome :icon="faRotate" class="w-3 h-3 animate-spin" />
            {{ $t('common.loading') }}
          </span>
        </div>

        <div class="flex flex-wrap items-center gap-1.5 lg:ml-auto">
          <span class="mono text-[9.5px] tracking-[0.12em] uppercase text-text-muted hidden sm:inline">{{
            $t('report.quick') }}</span>
          <button @click="preset('today')"
            class="px-3 min-h-[32px] rounded-full border text-[11.5px] font-bold mono transition"
            :class="rangeLabel === $t('report.rangeToday') ? 'bg-primary-deep text-white border-primary-deep' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
              $t('report.rangeToday') }}</button>
          <button @click="preset('yesterday')"
            class="px-3 min-h-[32px] rounded-full border text-[11.5px] font-bold mono transition"
            :class="rangeLabel === $t('report.rangeYesterday') ? 'bg-primary-deep text-white border-primary-deep' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
              $t('report.rangeYesterday') }}</button>
          <button @click="preset('7d')"
            class="px-3 min-h-[32px] rounded-full border text-[11.5px] font-bold mono transition"
            :class="rangeLabel === $t('report.range7long') ? 'bg-primary-deep text-white border-primary-deep' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
              $t('report.range7') }}</button>
          <button @click="preset('30d')"
            class="px-3 min-h-[32px] rounded-full border text-[11.5px] font-bold mono transition"
            :class="rangeLabel === $t('report.range30long') ? 'bg-primary-deep text-white border-primary-deep' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
              $t('report.range30') }}</button>
          <button @click="preset('month')"
            class="px-3 min-h-[32px] rounded-full border text-[11.5px] font-bold mono transition"
            :class="rangeLabel === $t('report.rangeMonth') ? 'bg-primary-deep text-white border-primary-deep' : 'bg-surface border-border-strong text-text-muted hover:bg-surface-raised'">{{
              $t('report.rangeMonth') }}</button>
        </div>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-12 gap-2.5">
        <div class="lg:col-span-5 relative">
          <font-awesome :icon="faMagnifyingGlass"
            class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
          <input v-model="filter" type="text" :placeholder="$t('report.searchPh')"
            class="field has-icon-left has-icon-right" :aria-label="$t('report.searchPh')">
          <button v-if="filter" @click="filter = ''"
            class="absolute right-1.5 top-1/2 -translate-y-1/2 w-9 h-9 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised transition"
            :aria-label="$t('common.clearSearch')">
            <font-awesome :icon="faXmark" class="w-3 h-3" />
          </button>
        </div>

        <div class="lg:col-span-3">
          <select v-model="deptFilter" class="field" :aria-label="$t('report.allDepartments')">
            <option value="">{{ $t('report.allDepartments') }}</option>
            <option v-for="d in depts" :key="d" :value="d">{{ d }}</option>
          </select>
        </div>

        <div class="lg:col-span-3">
          <select v-model="statusFilter" class="field" :aria-label="$t('report.allStatuses')">
            <option value="">{{ $t('report.allStatuses') }}</option>
            <option value="present">{{ $t('report.statusPresent') }}</option>
            <option value="late">{{ $t('report.statusLate') }}</option>
            <option value="early">{{ $t('report.statusEarlyLeave') }}</option>
            <option value="puncht">{{ $t('report.statusAbsent') }}</option>
            <option value="overtime">{{ $t('report.statusOvertime') }}</option>
            <option value="holiday">{{ $t('report.statusHoliday') }}</option>
          </select>
        </div>

        <div class="lg:col-span-1">
          <button @click="clearFilters()" class="btn btn-secondary w-full !px-2 text-[12px]">{{ $t('common.reset')
            }}</button>
        </div>
      </div>

      <div v-if="filter || deptFilter || statusFilter" class="flex flex-wrap items-center gap-1.5 mono text-[10.5px]">
        <span class="text-text-muted">{{ $t('common.activeFilters') }}</span>
        <span v-if="filter" class="chip chip-quiet">"{{ filter }}"</span>
        <span v-if="deptFilter" class="chip chip-quiet">{{ deptFilter }}</span>
        <span v-if="statusFilter" class="chip chip-quiet">
          {{ statusFilter === 'present' ? $t('report.statusPresent') : statusFilter === 'late' ? $t('report.statusLate') :
            statusFilter === 'early' ? $t('report.statusEarlyLeave') : statusFilter === 'puncht' ? $t('report.statusAbsent') :
              statusFilter === 'overtime' ? $t('report.statusOvertime') : $t('report.statusHoliday') }}
        </span>
        <span class="text-text-muted">{{ filteredCount }} / {{ allCount }}</span>
      </div>
    </div>

    <!-- ===== summary strip: one panel, six readings ===== -->
    <div class="panel grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 divide-x divide-border overflow-hidden">
      <div class="px-4 py-3">
        <p class="card-label">{{ $t('report.statPresent') }}</p>
        <p class="num text-[19px] text-text mt-0.5">{{ statPresent }}</p>
      </div>
      <div class="px-4 py-3 bg-surface-raised">
        <p class="card-label !text-primary-deeper">{{ $t('report.statOnTime') }}</p>
        <p class="display text-xl text-primary-deeper mt-0.5 num">{{ statPresent }}</p>
      </div>
      <div class="px-4 py-3">
        <p class="card-label">{{ $t('report.statLate') }}</p>
        <p class="num text-[19px] mt-0.5" :class="statLate > 0 ? 'text-danger' : 'text-text'">{{ statLate }}</p>
      </div>
      <div class="px-4 py-3">
        <p class="card-label">{{ $t('report.statAbsent') }}</p>
        <p class="num text-[19px] text-text mt-0.5">{{ statAbsent }}</p>
      </div>
      <div class="px-4 py-3">
        <p class="card-label">{{ $t('report.statEarly') }}</p>
        <p class="num text-[19px] mt-0.5" :class="statEarly > 0 ? 'text-warn' : 'text-text'">{{ statEarly }}</p>
      </div>
      <div class="px-4 py-3">
        <p class="card-label">{{ $t('report.statTotalHours') }}</p>
        <p class="num text-[19px] text-text mt-0.5">{{ fmtHMS(totalWorkingSec) }}</p>
      </div>
    </div>

    <!-- ===== rows ===== -->
    <div class="panel overflow-hidden">
      <div class="panel-head">
        <span class="inline-flex items-center gap-2">
          <span class="w-1.5 h-1.5 rounded-full" :class="loading ? 'bg-amber-500 data-tick' : 'bg-primary'"></span>
          {{ $t('report.dailySummary') }}
        </span>
        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted">{{ filteredCount }} / {{ allCount
          }}</span>
      </div>
      <ListReport :filter="filter" :dept-filter="deptFilter" :status-filter="statusFilter" />
    </div>

    <p class="mono text-[10px] text-text-muted px-1">{{ $t('report.activeRange') }} <b class="text-text">{{ rangeLabel
        }}</b></p>
  </div>
</template>
