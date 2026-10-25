<script setup>
import { useReportStore } from '~/store/report';
import { useSettingsStore } from '~/store/settings';
import { useChecker, useFormater } from '#imports';
import { faInbox } from '@fortawesome/free-solid-svg-icons';

const
  list = useReportStore(),
  settings = useSettingsStore(),
  check = useChecker(),
  format = useFormater(),
  props = defineProps({
    filter: { type: String, default: '' },
    deptFilter: { type: String, default: '' },
    statusFilter: { type: String, default: '' },
  });

const overtimeTol = computed(() => Number(settings.overtimeTolSec) || 0)

function statusTags(issue) {
  const tags = []
  const isAbsen = check.isNull(issue.start) && check.isNull(issue.end)
  if (issue.late_time > 0) tags.push('late')
  // Early/overtime are checkout-based: compare actual OUT against shift end
  // in the device's own timezone (not working_time vs duration - a late
  // arrival already shrinks working_time even when checkout is on time).
  if (!isAbsen && !check.isNull(issue.end)) {
    const tz = Number(issue.tz) || 0
    const endRef = (Number(issue.date) || 0) - tz * 3600 + (issue.shift_end ?? 0)
    // The early threshold follows the configured tolerance; the overtime tag uses
    // the `overtime` value the backend already computed with the same tolerance.
    if (Number(issue.end) < endRef - overtimeTol.value) tags.push('early')
  }
  if (Number(issue.overtime) > 0) tags.push('overtime')
  // A row counted on an included holiday: worked, but at a different pay rate.
  // Tagging it separately from late/overtime keeps payroll able to filter on the
  // DAY rather than on how this individual behaved.
  if (issue.holiday_name) tags.push('holiday')
  if (!isAbsen && check.isNull(issue.end)) tags.push('early')
  if (isAbsen) tags.push('puncht')
  if (tags.length === 0 && !isAbsen) tags.push('present')
  return tags
}
function hasTag(issue, tag) { return statusTags(issue).includes(tag) }

const contents = computed(() => {
  let arr = list.contents || []
  const q = (props.filter || '').trim().toLowerCase()
  if (q) arr = arr.filter(item => (`${item.first_name || ''} ${item.last_name || ''} ${item.department || ''}`).toLowerCase().includes(q))
  if (props.deptFilter) arr = arr.filter(item => (item.department || '') === props.deptFilter)
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
function fmtDate(ts) { return format.stamp_to_naive_date(ts) }
function fmtTime(ts, tz) {
  if (check.isNull(ts)) return '-'
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
  if (check.isNull(sec)) return '-'
  return format.sec_to_naive(sec)
}
function shiftDur(issue) {
  const s = issue.shift_start ?? 0
  const e = issue.shift_end ?? 0
  return Math.max(0, e >= s ? e - s : e - s + 86400)
}
function overtimeSec(issue) {
  // The backend already computes overtime seconds using `overtime_tolerance_sec`
  // from the settings table, so the frontend does not recompute it.
  return Number(issue.overtime) || 0
}
</script>

<template>
  <div class="overflow-auto max-h-[62vh] md:max-h-[58vh]">

    <!-- ===== desktop table ===== -->
    <table class="w-full min-w-[900px] border-collapse hidden md:table">
      <thead class="sticky top-0 z-10">
        <tr>
          <th class="th">{{ $t('report.colEmployee') }}</th>
          <th class="th">{{ $t('report.colDate') }}</th>
          <th class="th">{{ $t('report.colShift') }}</th>
          <th class="th">{{ $t('report.colPunch') }}</th>
          <th class="th">{{ $t('report.colStatus') }}</th>
          <th class="th !text-right">{{ $t('report.colDuration') }}</th>
        </tr>
      </thead>

      <tbody v-if="contents && contents.length">
        <tr v-for="(issue, idx) in contents" :key="`${issue.id}-${issue.date}-${idx}`"
          class="even:bg-surface-raised/25">
          <!-- employee -->
          <td class="td">
            <div class="flex items-center gap-2.5">
              <div
                class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center text-[11px] font-bold shrink-0">
                {{ initials(issue) }}
              </div>
              <div class="min-w-0">
                <p class="text-[13px] font-semibold text-text truncate">{{ issue.first_name }} {{ issue.last_name }}</p>
                <p class="mono text-[10px] text-text-muted truncate">
                  {{ issue.department || '-' }} <span class="opacity-60">#{{ issue.id }}</span>
                </p>
              </div>
            </div>
          </td>

          <!-- date -->
          <td class="td whitespace-nowrap">
            <p class="mono text-[11.5px] font-bold text-text">{{ fmtDate(issue.date) }}</p>
            <p class="mono text-[10px] text-text-muted">{{ fmtWday(issue.date) }}</p>
          </td>

          <!-- shift -->
          <td class="td">
            <p class="text-[12.5px] font-semibold text-text truncate">{{ issue.shift_name || '-' }}</p>
            <p class="mono text-[10.5px] text-text-muted whitespace-nowrap">
              {{ format.sec_to_naive(issue.shift_start) }} - {{ format.sec_to_naive(issue.shift_end) }}
              <span class="opacity-60">· {{ fmtDur(shiftDur(issue)) }}</span>
            </p>
            <p v-if="issue.schedule_name" class="mono text-[10px] text-text-muted/70 truncate">{{ issue.schedule_name }}
            </p>
          </td>

          <!-- in / out -->
          <td class="td whitespace-nowrap">
            <div class="flex items-center gap-2">
              <span class="mono text-[10px] font-bold text-text-muted w-12 shrink-0">{{ $t('report.colIn') }}</span>
              <span class="mono text-[12px] font-bold shrink-0"
                :class="check.isNull(issue.start) ? 'text-text-muted/60' : issue.late_time > 0 ? 'text-danger' : 'text-text'">
                {{ fmtTime(issue.start, issue.tz) }}
              </span>
            </div>
            <div class="flex items-center gap-2 mt-0.5">
              <span class="mono text-[10px] font-bold text-text-muted w-12 shrink-0">{{ $t('report.colOut') }}</span>
              <span class="mono text-[12px] font-bold shrink-0"
                :class="check.isNull(issue.end) ? 'text-text-muted/60' : 'text-text'">
                {{ fmtTime(issue.end, issue.tz) }}
              </span>
            </div>
            <p v-if="issue.enroll_device" class="mono text-[10px] text-text-muted/70 mt-0.5 truncate max-w-[170px]">
              {{ issue.enroll_device }}<span v-if="issue.tz !== undefined && issue.tz !== null"> · {{ tzShort(issue.tz)
                }}</span>
            </p>
          </td>

          <!-- status -->
          <td class="td">
            <div class="flex flex-wrap gap-1 max-w-[180px]">
              <span v-if="hasTag(issue, 'late')" class="chip chip-danger">{{ $t('report.statusLate') }}</span>
              <span v-if="hasTag(issue, 'early')" class="chip chip-warn">{{ $t('report.statusEarlyLeave') }}</span>
              <span v-if="hasTag(issue, 'puncht')" class="chip chip-danger">{{ $t('report.statusAbsent') }}</span>
              <span v-if="hasTag(issue, 'overtime')" class="chip chip-quiet">{{ $t('report.statusOvertime') }}</span>
              <span v-if="hasTag(issue, 'present')" class="chip chip-ok">{{ $t('report.statusPresent') }}</span>
              <span v-if="hasTag(issue, 'holiday')" class="chip chip-warn">{{ $t('report.statusHoliday') }}</span>
            </div>
          </td>

          <!-- duration -->
          <td class="td !text-right whitespace-nowrap">
            <p class="mono text-[12px] font-bold text-text">{{ fmtDur(issue.working_time) }}</p>
            <p v-if="issue.late_time > 0" class="mono text-[10px] font-bold text-danger">
              {{ $t('report.lateTag') }} {{ fmtDur(issue.late_time) }}
            </p>
            <p v-if="overtimeSec(issue) > 0" class="mono text-[10px] font-bold text-primary-deep">
              {{ $t('report.overtimeTag') }} {{ fmtDur(overtimeSec(issue)) }}
            </p>
          </td>
        </tr>
      </tbody>

      <tbody v-else>
        <tr>
          <td colspan="6" class="p-0">
            <div class="state">
              <div class="state-icon">
                <span class="hud-corner hud-corner-tl"></span>
                <span class="hud-corner hud-corner-br"></span>
                <font-awesome :icon="faInbox" class="w-5 h-5" />
              </div>
              <p class="state-title">{{ $t('employee.noData') }}</p>
              <p class="state-hint">{{ $t('report.emptyHint') }}</p>
            </div>
          </td>
        </tr>
      </tbody>
    </table>

    <!-- ===== mobile cards ===== -->
    <div class="md:hidden p-3 space-y-2.5" v-if="contents && contents.length">
      <div v-for="(issue, idx) in contents" :key="`m-${issue.id}-${issue.date}-${idx}`"
        class="rounded-xl border border-border bg-surface p-3">
        <div class="flex items-start justify-between gap-3">
          <div class="flex items-center gap-2.5 min-w-0">
            <div
              class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center text-[11px] font-bold shrink-0">
              {{ initials(issue) }}</div>
            <div class="min-w-0">
              <p class="text-[13px] font-semibold text-text truncate">{{ issue.first_name }} {{ issue.last_name }}</p>
              <p class="mono text-[10px] text-text-muted truncate">{{ issue.department || '-' }} <span
                  class="opacity-60">#{{ issue.id }}</span></p>
            </div>
          </div>
          <span class="chip chip-quiet shrink-0">{{ issue.shift_name || '-' }}</span>
        </div>

        <div class="mt-2.5 grid grid-cols-2 gap-2">
          <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
            <p class="mono text-[9.5px] tracking-[0.1em] uppercase text-text-muted">{{ $t('report.colDate') }}</p>
            <p class="mono text-[11.5px] font-bold text-text mt-0.5">{{ fmtDate(issue.date) }}</p>
            <p class="mono text-[10px] text-text-muted">{{ fmtWday(issue.date) }}</p>
          </div>
          <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
            <p class="mono text-[9.5px] tracking-[0.1em] uppercase text-text-muted">{{ $t('report.colPunch') }}</p>
            <p class="mono text-[11.5px] font-bold text-text mt-0.5">
              <span class="text-text-muted text-[9.5px]">{{ $t('report.colIn') }}</span> {{ fmtTime(issue.start,
              issue.tz) }}
            </p>
            <p class="mono text-[11.5px] font-bold text-text">
              <span class="text-text-muted text-[9.5px]">{{ $t('report.colOut') }}</span> {{ fmtTime(issue.end,
              issue.tz) }}
            </p>
          </div>
        </div>

        <div class="mt-2 flex flex-wrap gap-1">
          <span v-if="hasTag(issue, 'late')" class="chip chip-danger">{{ $t('report.statusLate') }}</span>
          <span v-if="hasTag(issue, 'early')" class="chip chip-warn">{{ $t('report.statusEarlyLeave') }}</span>
          <span v-if="hasTag(issue, 'puncht')" class="chip chip-danger">{{ $t('report.statusAbsent') }}</span>
          <span v-if="hasTag(issue, 'overtime')" class="chip chip-quiet">{{ $t('report.statusOvertime') }}</span>
          <span v-if="hasTag(issue, 'present')" class="chip chip-ok">{{ $t('report.statusPresent') }}</span>
          <span v-if="hasTag(issue, 'holiday')" class="chip chip-warn">{{ $t('report.statusHoliday') }}</span>
        </div>

        <div class="mt-2 flex items-center justify-between mono text-[11px] border-t border-border pt-2">
          <span class="text-text-muted">{{ $t('report.colWorked') }} <b class="text-text">{{ fmtDur(issue.working_time)
              }}</b></span>
          <span class="flex items-center gap-1.5">
            <b v-if="overtimeSec(issue) > 0" class="text-primary-deep">+{{ fmtDur(overtimeSec(issue)) }}</b>
            <b v-if="issue.late_time > 0" class="text-danger">-{{ fmtDur(issue.late_time) }}</b>
          </span>
        </div>
      </div>
    </div>

    <div v-else class="md:hidden">
      <div class="state">
        <div class="state-icon"><font-awesome :icon="faInbox" class="w-5 h-5" /></div>
        <p class="state-title">{{ $t('employee.noData') }}</p>
        <p class="state-hint">{{ $t('report.emptyHint') }}</p>
      </div>
    </div>

    <div v-if="contents && contents.length"
      class="sticky bottom-0 bg-surface/95 backdrop-blur border-t border-border px-4 py-2 flex items-center justify-between mono text-[10.5px]">
      <span class="text-text-muted">{{ $t('report.rowsFiltered', {
        shown: contents.length, total:
          (list.contents || []).length }) }}</span>
    </div>
  </div>
</template>
