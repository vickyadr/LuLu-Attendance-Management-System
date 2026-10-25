<script setup>
import { useTransaction } from '~/store/transaction';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';
import { faInbox } from '@fortawesome/free-solid-svg-icons';

const
  auth = useAuthStore(),
  config = useRuntimeConfig(),
  list = useTransaction(),
  last_data = ref(0),
  check = useChecker(),
  format = useFormater(),
  contents = computed(() => {
    return list.contents
  });

// The dashboard page owns the WebSocket and passes the status down, so the
// page can react to the same events. `refresh` lets it tell this table to
// re-read when a device upload arrives; `0` means "fetch once on mount" (used
// when the table is rendered outside a live feed). Runtime props, not
// type-only defineProps: this file is plain JS, where the TS form does not parse.
const props = defineProps({
  wsStatus: { type: String, default: 'closed' },
  refresh: { type: Number, default: 0 },
})
const wsStatus = computed(() => props.wsStatus)

async function initLiveTransaction() {
  await apiFetch(`${config.public.apiBase}/transaction/live`,
    {
      method: "GET",
      headers: auth.confHeaders(),
    }).then(async (response) => {
      if (response.code == 200) {
        list.set(response.data);
        last_data.value = parseInt(response.message);
      }
    });
};

// Watch the counter instead of opening a second socket here: two connections
// per page would double the backend sessions and let the two drift apart.
watch(() => props.refresh, () => initLiveTransaction());

function parseEnrollType(input) {
  // Index 5 is a wire code LuLu does not map (an operation or verification
  // modality outside the documented table). The row is kept rather than
  // dropped, so it needs a label -- `inSwitch` returns undefined past the end
  // of the array and the chip would render empty.
  return check.inSwitch(input, [$t('live.typeUnknown'), $t('live.typeFinger'), $t('live.typePin'), $t('live.typeCard'), $t('live.typeFace'), $t('live.typeOther')])
}
function initials(a, b) {
  return `${(a || '')[0] || ''}${(b || '')[0] || ''}`.toUpperCase().slice(0, 2) || '•'
}
function chipCls(t) {
  const v = parseEnrollType(t)
  if (v === 'Finger') return 'bg-emerald-50 border-emerald-200 text-emerald-700'
  if (v === 'Face') return 'bg-lime-50 border-lime-200 text-lime-700'
  if (v === 'Card') return 'bg-sky-50 border-sky-200 text-sky-700'
  if (v === 'PIN') return 'bg-amber-50 border-amber-200 text-amber-700'
  return 'bg-emerald-50 border-emerald-200 text-emerald-800'
}
function dotCls(t) {
  const v = parseEnrollType(t)
  if (v === 'Finger') return 'bg-emerald-500'
  if (v === 'Face') return 'bg-lime-500'
  if (v === 'Card') return 'bg-sky-500'
  if (v === 'PIN') return 'bg-amber-500'
  return 'bg-slate-400'
}
function tzShort(tz) {
  const n = Number(tz)
  if (Number.isNaN(n)) return ''
  return `GMT${n >= 0 ? '+' : ''}${n}`
}
function timeParts(ts, tz) {
  const n = Number(tz)
  // Device TZ render: enroll_time is a UTC epoch, so shift by the device's own
  // timezone and read back with UTC getters - independent of browser locale.
  if (!Number.isNaN(n)) {
    const d = new Date((Number(ts) + n * 3600) * 1000)
    const p = (v) => String(v).padStart(2, '0')
    return {
      d: `${d.getUTCFullYear()}-${p(d.getUTCMonth() + 1)}-${p(d.getUTCDate())}`,
      h: `${p(d.getUTCHours())}:${p(d.getUTCMinutes())}:${p(d.getUTCSeconds())}`
    }
  }
  const s = format.stamp_to_naive(ts) // "YYYY-MM-DD HH:MM:SS"
  const [d, h] = s.split(' ')
  return { d: d || '', h: h || '' }
}

onMounted(() => {
  initLiveTransaction()
});
</script>

<template>
  <div class="panel overflow-hidden flex flex-col min-h-0 max-h-[62dvh] sm:max-h-[68vh] lg:max-h-none lg:flex-1">

    <!-- Desktop header -->
    <div class="hidden sm:flex items-center justify-between gap-3 px-4 py-2.5 border-b border-border shrink-0">
      <div class="flex items-center gap-2.5 min-w-0">
        <span class="w-1.5 h-1.5 rounded-full bg-primary data-tick shrink-0"></span>
        <h2 class="display text-[13.5px] text-text truncate">{{ $t('live.title') }}</h2>
        <span class="chip chip-lime">{{ $t('dashboard.liveFeed') }}</span>
        <span class="mono text-[10.5px] text-text-muted hidden md:inline num">{{ contents.length }} {{
          $t('dashboard.records') }}</span>
      </div>
      <span class="mono text-[10px] tracking-[0.1em] text-text-muted inline-flex items-center gap-1.5 shrink-0">
        <span class="w-1.5 h-1.5 rounded-full"
          :class="wsStatus === 'open' ? 'bg-primary' : wsStatus === 'reconnecting' ? 'bg-amber-500' : 'bg-red-400'"></span>
        <template v-if="wsStatus === 'open'">{{ $t('live.connected') }}</template>
        <template v-else-if="wsStatus === 'reconnecting'">{{ $t('dashboard.reconnecting') }}</template>
        <template v-else>{{ $t('live.offline') }}</template>
      </span>
    </div>

    <!-- Mobile header -->
    <div class="sm:hidden flex items-center justify-between gap-2 px-3 py-2.5 border-b border-border shrink-0">
      <span class="inline-flex items-center gap-1.5 min-w-0">
        <span class="w-1.5 h-1.5 rounded-full bg-primary data-tick shrink-0"></span>
        <span class="display text-[12.5px] text-text truncate">{{ $t('live.title') }}</span>
      </span>
      <span class="mono text-[10px] text-text-muted shrink-0 num">{{ contents.length }}</span>
    </div>

    <!-- Table -->
    <div class="table-shell hidden sm:block flex-1 min-h-0 lg:flex-1 lg:min-h-0">
      <table class="w-full min-w-[620px] border-collapse">
        <thead>
          <tr>
            <th class="th w-[22%]">{{ $t('live.colTime') }}</th>
            <th class="th w-[30%]">{{ $t('live.colEmployee') }}</th>
            <th class="th w-[18%]">{{ $t('live.colDepartment') }}</th>
            <th class="th w-[14%] !text-center">{{ $t('live.colMethod') }}</th>
            <th class="th w-[16%]">{{ $t('live.colLocation') }}</th>
          </tr>
        </thead>

        <tbody v-if="contents.length > 0">
          <tr v-for="(enroll, index) in contents" :key="enroll.id ?? index" class="even:bg-surface-raised/25">
            <td class="td whitespace-nowrap">
              <div class="flex items-center gap-2">
                <span class="mono text-[11px] text-text-muted">{{ timeParts(enroll.date_time, enroll.tz).d }}</span>
                <span class="mono text-[11.5px] font-bold text-text">{{ timeParts(enroll.date_time, enroll.tz).h
                }}</span>
                <span v-if="tzShort(enroll.tz)" class="mono text-[10px] text-text-muted/70">{{ tzShort(enroll.tz)
                }}</span>
              </div>
            </td>
            <td class="td whitespace-nowrap">
              <div class="flex items-center gap-2.5">
                <div
                  class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center text-[11px] font-bold shrink-0">
                  {{ initials(enroll.first_name, enroll.last_name) }}
                </div>
                <div class="leading-tight min-w-0">
                  <p class="text-[13px] font-semibold text-text truncate">{{ enroll.first_name }} {{ enroll.last_name }}
                  </p>
                  <p class="mono text-[10px] text-text-muted">ID #{{ enroll.employee_id ?? '-' }}</p>
                </div>
              </div>
            </td>
            <td class="td whitespace-nowrap">
              <span v-if="enroll.department" class="chip chip-quiet">{{ enroll.department }}</span>
              <span v-else class="mono text-[11px] text-text-muted">-</span>
            </td>
            <td class="td !text-center whitespace-nowrap">
              <span class="inline-flex items-center gap-1.5 mono text-[11px] font-bold text-text-muted">
                <span class="w-1.5 h-1.5 rounded-full" :class="dotCls(enroll.enroll_type)"></span>
                {{ parseEnrollType(enroll.enroll_type) }}
              </span>
            </td>
            <td class="td whitespace-nowrap">
              <span class="mono text-[11px] text-text-muted truncate">{{ enroll.location }}</span>
            </td>
          </tr>
        </tbody>

        <tbody v-else>
          <tr>
            <td colspan="5" class="p-0">
              <div class="state">
                <div class="state-icon">
                  <span class="hud-corner hud-corner-tl"></span>
                  <span class="hud-corner hud-corner-br"></span>
                  <font-awesome :icon="faInbox" class="w-5 h-5" />
                </div>
                <p class="state-title">{{ $t('live.empty') }}</p>
                <span class="chip">
                  <span class="w-1.5 h-1.5 rounded-full"
                    :class="wsStatus === 'open' ? 'bg-primary data-tick' : 'bg-amber-500'"></span>
                  <template v-if="wsStatus === 'open'">{{ $t('live.connected') }}</template>
                  <template v-else-if="wsStatus === 'reconnecting'">{{ $t('dashboard.reconnecting') }}</template>
                  <template v-else>{{ $t('live.offline') }}</template>
                </span>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Mobile cards -->
    <div class="sm:hidden p-2 space-y-2 overflow-y-auto max-h-[62dvh]">
      <div v-if="contents.length === 0" class="state !py-8">
        <div class="state-icon !w-11 !h-11">
          <font-awesome :icon="faInbox" class="w-4 h-4" />
        </div>
        <p class="state-title">{{ $t('live.empty') }}</p>
      </div>

      <div v-for="(enroll, index) in contents" :key="enroll.id ?? index"
        class="rounded-xl border border-border bg-surface p-3 flex items-center gap-3">
        <div
          class="w-9 h-9 rounded-lg bg-surface-raised text-primary-deeper grid place-content-center text-[11px] font-bold shrink-0">
          {{ initials(enroll.first_name, enroll.last_name) }}
        </div>
        <div class="flex-1 min-w-0">
          <p class="font-semibold text-[13px] text-text truncate">{{ enroll.first_name }} {{ enroll.last_name }}</p>
          <p class="mono text-[10.5px] text-text-muted truncate">
            {{ timeParts(enroll.date_time, enroll.tz).h }}
            <span v-if="tzShort(enroll.tz)">{{ tzShort(enroll.tz) }}</span>
            <span v-if="enroll.department"> · {{ enroll.department }}</span>
          </p>
        </div>
        <span class="inline-flex items-center gap-1.5 mono text-[10.5px] font-bold text-text-muted shrink-0">
          <span class="w-1.5 h-1.5 rounded-full" :class="dotCls(enroll.enroll_type)"></span>
          {{ parseEnrollType(enroll.enroll_type) }}
        </span>
      </div>
    </div>

  </div>
</template>
