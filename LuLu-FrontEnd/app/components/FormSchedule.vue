<script setup>
import { useChecker, useFormater } from '#imports';
import { faRotate } from '@fortawesome/free-solid-svg-icons';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY_LABEL } from '~/utils/labels';
import { useSchedule } from '~/store/schedule';
import { useNotifyStore } from '~/store/notify';

const { t } = useI18n()

const
  auth = useAuthStore(),
  // /schedule/* is gated on the 'shift' page server-side; the UI must agree.
  canAdd = computed(() => auth.hasActionAccess('shift', 'can_add')),
  canEdit = computed(() => auth.hasActionAccess('shift', 'can_edit')),
  notify = useNotifyStore(),
  schedule = useSchedule(),
  shift = useShiftStore(),
  config = useRuntimeConfig(),
  check = useChecker(),
  emit = defineEmits(['done']),
  validator = reactive({ name: '', pattern: '', shift: '' }),
  form_data = reactive({
    name: '',
    pattern: 1,
    shift: [null],
  });

// options: label + value = schedule_type
const patternOptions = computed(() => [
  { label: t('schedule.patternDaily'), value: 1, desc: t('schedule.descFlat'), badge: t('schedule.badgeDaily') },
  { label: t('schedule.patternWeekly'), value: 7, desc: t('schedule.descDays7'), badge: t('schedule.badgeWeekly') },
  { label: t('schedule.patternWeeks2'), value: 14, desc: t('schedule.descDays14'), badge: t('schedule.weeks2') },
  { label: t('schedule.patternWeeks3'), value: 21, desc: t('schedule.descDays21'), badge: t('schedule.weeks3') },
  { label: t('schedule.patternMonthly'), value: 28, desc: t('schedule.descDays28'), badge: t('schedule.badgeMonthly') },
])

function patternMeta(v) {
  return patternOptions.value.find(o => o.value === v) || patternOptions.value[0]
}

const days = computed(() => [0, 1, 2, 3, 4, 5, 6].map(i => t(`schedule.day${i}`)))
function dayLabel(idx) {
  const d = idx % 7
  const week = Math.floor(idx / 7) + 1
  const base = days.value[d]
  if (form_data.pattern === 1) return base
  if (form_data.pattern === 7) return base
  return `${t('schedule.weekShort', { n: week })} - ${base}`
}
function shortDay(idx) { return days.value[idx % 7].slice(0, 3) }

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
    const r = await apiFetch(`${config.public.apiBase}/shift/list`, { method: 'GET', headers: auth.confHeaders() })
    if (r.code === 200) shift.set(r.data)
  } catch (e) { console.error(e) }
}
onMounted(() => { ensureShiftsLoaded(); ensureShiftLength() })

function validate() {
  validator.name = ''; validator.pattern = ''; validator.shift = ''
  if (!form_data.name || form_data.name.trim().length < 3) validator.name = t('schedule.errNameShort')
  if (!patternOptions.value.some(o => o.value === form_data.pattern)) validator.pattern = t('schedule.errPattern')
  const n = form_data.pattern
  if (form_data.shift.length !== n) validator.shift = t('schedule.errShiftCount', { n })
  else if (form_data.shift.some(v => v === null || v === undefined || v === '')) validator.shift = t('schedule.errFillDays')
  return !validator.name && !validator.pattern && !validator.shift
}


const submitting = ref(false)
const add_schedule = async () => {
  // The button is hidden without permission; this also covers the form-submit path.
  if (!canAdd.value) { notify.warning(t('common.noAccess'), t('device.noPermission')); return }
  if (!validate()) return
  submitting.value = true
  validator.shift = ''
  try {
    const body = {
      name: form_data.name.trim(),
      pattern: form_data.pattern,
      shift: form_data.shift.map(v => v === 0 || v === '0' ? 0 : Number(v)),
    }
    const response = await apiFetch(`${config.public.apiBase}/schedule/add`, { method: 'POST', body, headers: auth.confHeaders() })
    if (response.code === 200) {
      // reload list
      try {
        const r2 = await apiFetch(`${config.public.apiBase}/schedule/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r2.code === 200) schedule.set(r2.data)
      } catch { }
      form_data.name = ''; clearShifts()
      ensureShiftLength()
      notify.success(t('schedule.createTitle'), response.message || t('schedule.created'))
      emit('done')
    } else {
      if (response.data) {
        validator.name = response.data.name || ''
        validator.shift = response.data.shift || response.data.pattern || ''
      }
      notify.error(t('common.saveFailed'), response.message || t('schedule.saveFailed'))
    }
  } catch (e) {
    const msg = e?.data?.message || e?.message || 'Connection failed'
    // try parse field errors
    if (e?.data?.data) {
      validator.name = e.data.data.name || ''
      validator.shift = e.data.data.shift || ''
    }
    notify.error(t('common.connFailed'), msg)
  } finally { submitting.value = false }
};
</script>

<template>
  <div class="text-left">
    <form @submit.prevent="add_schedule" class="panel-body space-y-4">

      <!-- context chips -->
      <div class="flex flex-wrap items-center gap-1.5">
        <span class="chip chip-ok">{{ patternMeta(form_data.pattern).badge }}</span>
      </div>

      <!-- name + pattern -->
      <div class="grid grid-cols-1 xl:grid-cols-2 gap-3">
        <div>
          <label for="schedule_name" class="label">{{ $t('schedule.name') }} <span class="text-danger">*</span></label>
          <input id="schedule_name" v-model="form_data.name" type="text" required :placeholder="$t('schedule.namePh')"
            class="field" :aria-invalid="!!validator.name">
          <p v-if="validator.name" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{ validator.name }}
          </p>
          <p v-else class="mt-1 mono text-[10px] text-text-muted">{{ $t('schedule.uniqueName') }}</p>
        </div>
        <div>
          <label for="schedule_pattern" class="label">{{ $t('schedule.pattern') }} <span
              class="text-danger">*</span></label>
          <select id="schedule_pattern" v-model.number="form_data.pattern" class="field"
            :aria-invalid="!!validator.pattern">
            <option v-for="o in patternOptions" :key="o.value" :value="o.value">{{ o.label }}</option>
          </select>
          <p v-if="validator.pattern" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
            validator.pattern }}</p>
          <p v-else class="mt-1 mono text-[10px] text-text-muted">{{ patternMeta(form_data.pattern).desc }} · {{
            $t('schedule.typeLabel', { n: form_data.pattern }) }}</p>
        </div>
      </div>

      <!-- info bar -->
      <div class="rounded-xl bg-surface-raised/60 border border-border px-3 py-2.5 flex flex-wrap items-center gap-2">
        <span class="chip chip-quiet">
          <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
          {{ $t('schedule.daysCount', { n: form_data.pattern }) }}
        </span>
        <span class="mono text-[10px] text-text-muted">{{ form_data.pattern === 1 ? $t('schedule.flatAllDays') :
          $t('schedule.weeksSlots', { weeks: form_data.pattern / 7, slots: form_data.pattern }) }}</span>
        <span class="ml-auto flex items-center gap-1.5">
          <button type="button" @click="clearShifts" class="btn btn-secondary">{{ $t('common.clear') }}</button>
          <button type="button" @click="fillAllWithFirst" class="btn btn-primary">{{ $t('schedule.copyWeek1All')
            }}</button>
        </span>
      </div>

      <!-- FLAT: one shift for every day -->
      <div v-if="form_data.pattern === 1" class="rounded-xl border border-border p-3.5 space-y-2">
        <p class="card-label !text-primary-deeper">{{ $t('schedule.dailyOneShift') }}</p>
        <p class="label">{{ $t('schedule.pickShift') }} <span class="text-danger">*</span></p>
        <select v-model="form_data.shift[0]" class="field" :aria-label="$t('schedule.pickShift')">
          <option :value="null" disabled>- {{ $t('schedule.pickShiftPh') }} -</option>
          <option :value="0">{{ $t('schedule.dayOff') }}</option>
          <option v-for="d in shift.contents" :key="d.id" :value="d.id">{{ d.name }} - {{ d.start_time != null ?
            String(Math.floor(d.start_time / 3600)).padStart(2, '0') + ':' + String(Math.floor((d.start_time % 3600) / 60)).padStart(2,'0')
            : '' }} → {{ d.end_time != null ?
              String(Math.floor(d.end_time / 3600)).padStart(2, '0') + ':' + String(Math.floor((d.end_time % 3600) / 60)).padStart(2,'0')
            : '' }}</option>
        </select>
        <p v-if="shift.contents.length === 0"
          class="rounded-lg bg-warn-bg border border-amber-200 text-amber-700 text-[11.5px] px-3 py-2">{{
            $t('schedule.emptyShiftsFirst') }}</p>
      </div>

      <!-- WEEKLY / BIWEEKLY / MONTHLY -->
      <div v-else class="rounded-xl border border-border overflow-hidden">

        <div class="px-3 py-2.5 border-b border-border bg-surface-raised/60 flex flex-wrap items-center gap-2">
          <p class="card-label !text-primary-deeper">{{ $t('schedule.customPerDay') }}</p>
          <span class="ml-auto mono text-[10px] text-text-muted">{{ $t('schedule.pickPerDay') }}</span>
        </div>

        <div class="p-3 space-y-3">
          <div v-for="week in (form_data.pattern / 7)" :key="week"
            class="rounded-xl border border-border bg-surface p-2.5">
            <div class="flex items-center justify-between gap-2 mb-2">
              <span class="mono text-[10px] font-bold tracking-[0.14em] uppercase text-text-muted">{{
                $t('schedule.weekLabel', { n: week }) }}</span>
              <span class="flex items-center gap-1.5">
                <button v-if="week === 1 && form_data.pattern > 7" type="button" @click="copyWeek(1)"
                  class="btn btn-secondary">{{ $t('schedule.copyWeek1All') }}</button>
                <span class="mono text-[10px] text-text-muted/70 hidden sm:inline">{{ $t('schedule.days7Short')
                  }}</span>
              </span>
            </div>

            <div class="grid grid-cols-2 sm:grid-cols-3 gap-2">
              <div v-for="d in 7" :key="d" class="space-y-1">
                <label class="block mono text-[9.5px] uppercase tracking-[0.08em] text-text-muted ml-1">
                  {{ days[d - 1] }} <span class="text-text-muted/60">· {{ $t('schedule.weekShort', { n: week }) }}</span>
                </label>
                <select :value="form_data.shift[(week - 1) * 7 + (d - 1)]"
                  @change="form_data.shift[(week - 1) * 7 + (d - 1)] = $event.target.value === '' ? null : ($event.target.value === '0' ? 0 : Number($event.target.value))"
                  class="field !px-2 !text-[12px]"
                  :class="(form_data.shift[(week - 1) * 7 + (d - 1)] === null || form_data.shift[(week - 1) * 7 + (d - 1)] === '') ? '!border-amber-300 !bg-warn-bg' : ''">
                  <option :value="null" disabled>- {{ $t('schedule.pickDayPh') }} -</option>
                  <option :value="0">{{ OFF_DAY_LABEL }}</option>
                  <option v-for="s in shift.contents" :key="s.id" :value="s.id">{{ s.name }}</option>
                </select>
              </div>
            </div>
          </div>

          <p v-if="validator.shift" role="alert" class="text-[11.5px] font-medium text-danger ml-1">{{ validator.shift
            }}</p>
          <div v-if="shift.contents.length === 0"
            class="rounded-xl bg-warn-bg border border-amber-200 text-amber-700 text-[11.5px] px-3 py-2.5">{{
              $t('schedule.emptyShiftsFirst') }}</div>

          <!-- summary -->
          <div class="rounded-xl bg-canvas border border-border px-3 py-2.5">
            <p class="card-label mb-2">{{ $t('schedule.summaryPreview') }}</p>
            <div class="flex flex-wrap gap-1.5">
              <span v-for="(v, idx) in form_data.shift" :key="idx" class="chip"
                :class="v === null || v === '' ? 'chip-warn' : isHoliday(v) ? 'chip-quiet' : 'chip-ok'">
                <span class="w-1.5 h-1.5 rounded-full"
                  :class="v === null || v === '' ? 'bg-amber-500' : isHoliday(v) ? 'bg-text-muted/40' : 'bg-primary'"></span>
                {{ shortDay(idx % 7) }}{{ form_data.pattern > 7 ? ' ' + $t('schedule.weekShort', { n: Math.floor(idx / 7) + 1 }) : ''
                }}: {{isHoliday(v) ? OFF_DAY_LABEL : (shift.contents.find(s => String(s.id) === String(v))?.name ||
                  (v ===null?'-':'?')) }}
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- actions -->
      <div class="flex flex-wrap items-center gap-2 pt-1">
        <button v-if="canAdd" type="submit" :disabled="submitting" class="btn btn-primary flex-1 sm:flex-none sm:px-7">
          <font-awesome v-if="submitting" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
          {{ submitting ? $t('common.saving') : $t('schedule.saveSchedule') }}
        </button>
        <button type="button" @click="form_data.name = ''; clearShifts(); ensureShiftLength()"
          class="btn btn-secondary flex-1 sm:flex-none">{{ $t('common.reset') }}</button>
      </div>
    </form>
  </div>
</template>
