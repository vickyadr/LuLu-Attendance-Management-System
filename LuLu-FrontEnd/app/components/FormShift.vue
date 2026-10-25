<script setup>
import { useChecker, useFormater } from '#imports';
import { faRotate } from '@fortawesome/free-solid-svg-icons';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift';
import { useNotifyStore } from '~/store/notify';

const { t } = useI18n()

const auth = useAuthStore(),
  // Writes are gated per action; /shift/* maps to the 'shift' page server-side.
  canAdd = computed(() => auth.hasActionAccess('shift', 'can_add')),
  canEdit = computed(() => auth.hasActionAccess('shift', 'can_edit')),
  canDelete = computed(() => auth.hasActionAccess('shift', 'can_delete')),
  notify = useNotifyStore(),
  shift_store = useShiftStore(),
  config = useRuntimeConfig(),
  format = useFormater(),
  check = useChecker(),
  shift = reactive({
    id: 0,
    name: '',
    start_time_str: '08:00',
    end_time_str: '17:00',
    start_enroll_min: 30,
    end_enroll_min: 30,
  }),
  validator = reactive({
    name: '', start_time: '', end_time: '', start_enroll: '', end_enroll: '',
  }),
  emit = defineEmits(['done-edit']);

const isEdit = computed(() => shift.id > 0)
const submitting = ref(false)

function callbackDoneEdit() { shift.id = 0; emit('done-edit') }

// --- helpers time ---
function secToTimeStr(sec) {
  let s = ((sec % 86400) + 86400) % 86400
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}
function timeStrToSec(str) {
  if (!str || !str.includes(':')) return 0
  const [h, m] = str.split(':').map(Number)
  return (Number.isFinite(h) ? h : 0) * 3600 + (Number.isFinite(m) ? m : 0) * 60
}
function fmtHM(sec) {
  const s = ((sec % 86400) + 86400) % 86400
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}

const startSec = computed(() => timeStrToSec(shift.start_time_str))
const endSec = computed(() => timeStrToSec(shift.end_time_str))
const durationMin = computed(() => {
  let d = endSec.value - startSec.value
  if (d <= 0) d += 86400
  return Math.round(d / 60)
})
const durationLabel = computed(() => {
  const h = Math.floor(durationMin.value / 60), m = durationMin.value % 60
  return `${h}h ${m}m`
})

// presets
const presets = [
  { label: $t('shift.presetPagi'), sub: '08:00 → 17:00', s: '08:00', e: '17:00', enroll: 30 },
  { label: $t('shift.presetSiang'), sub: '14:00 → 22:00', s: '14:00', e: '22:00', enroll: 30 },
  { label: $t('shift.presetMalam'), sub: '22:00 → 06:00', s: '22:00', e: '06:00', enroll: 30 },
  { label: $t('shift.presetFleksibel'), sub: '07:00 → 15:00', s: '07:00', e: '15:00', enroll: 15 },
]
function applyPreset(p) {
  shift.start_time_str = p.s
  shift.end_time_str = p.e
  shift.start_enroll_min = p.enroll
  shift.end_enroll_min = p.enroll
}

async function add_shift() {
  if (!canAdd.value) {
    // UI hides the control; this stops a programmatic call reaching a 403.
    notify.warning(t('common.noAccess'), t('device.noPermission'));
    return
  }
  submitting.value = true
  validator.name = ''; validator.start_time = ''; validator.end_time = ''; validator.start_enroll = ''; validator.end_enroll = ''
  try {
    const body = {
      name: shift.name.trim(),
      start_time: timeStrToSec(shift.start_time_str),
      end_time: timeStrToSec(shift.end_time_str),
      start_enroll: Number(shift.start_enroll_min) * 60,
      end_enroll: Number(shift.end_enroll_min) * 60,
    }
    if (!body.name || body.name.length < 3) { validator.name = t('shift.errNameShort'); submitting.value = false; return }
    const response = await apiFetch(`${config.public.apiBase}/shift/add`, { body, method: 'POST', headers: auth.confHeaders() })
    if (response.code == 200) shift_store.addList(response.data[0]);
    else {
      validator.end_enroll = response.data?.end_enroll || ''
      validator.end_time = response.data?.end_time || ''
      validator.name = response.data?.name || ''
      validator.start_enroll = response.data?.start_enroll || ''
      validator.start_time = response.data?.start_time || ''
    }
    notify[response.code == 200 ? 'success' : 'error'](t('shift.addTitle'), response.message)
    if (response.code == 200) { shift.name = ''; /* keep times */ }
  } catch (e) {
    const msg = e?.data?.message || e?.message || t('common.connectionFailed')
    notify.error(t('common.connFailed'), msg)
  } finally { submitting.value = false }
}

async function edit_shift() {
  if (!canEdit.value) {
    // UI hides the control; this stops a programmatic call reaching a 403.
    notify.warning(t('common.noAccess'), t('device.noPermission'));
    return
  }
  submitting.value = true
  validator.name = ''; validator.start_time = ''; validator.end_time = ''; validator.start_enroll = ''; validator.end_enroll = ''
  try {
    const body = {
      id: shift.id,
      name: shift.name.trim(),
      start_time: timeStrToSec(shift.start_time_str),
      end_time: timeStrToSec(shift.end_time_str),
      start_enroll: Number(shift.start_enroll_min) * 60,
      end_enroll: Number(shift.end_enroll_min) * 60,
    }
    const response = await apiFetch(`${config.public.apiBase}/shift/edit`, { body, method: 'POST', headers: auth.confHeaders() })
    if (response.code == 200) {
      shift_store.updateList(response.data[0]);
      shift.id = 0; callbackDoneEdit();
    } else {
      validator.end_enroll = response.data?.end_enroll || ''
      validator.end_time = response.data?.end_time || ''
      validator.name = response.data?.name || ''
      validator.start_enroll = response.data?.start_enroll || ''
      validator.start_time = response.data?.start_time || ''
    }
    notify[response.code == 200 ? 'success' : 'error'](t('shift.editTitle'), response.message)
  } catch (e) {
    notify.error(t('common.connFailed'), e?.data?.message || e?.message || t('common.connFailed'))
  } finally { submitting.value = false }
}

async function del_shift(id) {
  if (!canDelete.value) {
    // UI hides the control; this stops a programmatic call reaching a 403.
    notify.warning(t('common.noAccess'), t('device.noPermission'));
    return
  }
  try {
    const response = await apiFetch(`${config.public.apiBase}/shift/delete/${id}`, { method: 'GET', headers: auth.confHeaders() })
    if (response.code == 200) shift_store.removeList(id)
    notify[response.code == 200 ? 'success' : 'error'](t('shift.deleteTitle'), response.message)
  } catch (e) {
    notify.error(t('common.deleteFailed'), e?.data?.message || e?.message)
  }
}

function fillForm(data) {
  shift.id = data.id
  shift.name = data.name || ''
  // data.start_time / end_time are seconds; start_enroll/end_enroll are absolute seconds
  const st = Number(data.start_time ?? 0)
  const et = Number(data.end_time ?? 0)
  const se = Number(data.start_enroll ?? st)
  const ee = Number(data.end_enroll ?? et)
  shift.start_time_str = secToTimeStr(st)
  shift.end_time_str = secToTimeStr(et)
  // offset minutes
  let offStart = Math.round((st - se) / 60)
  let offEnd = Math.round((ee - et) / 60)
  // normalize negative/overflow to positive minutes within day
  if (offStart < 0) offStart = Math.round(((st + 86400 - se) % 86400) / 60)
  if (offEnd < 0) offEnd = Math.round(((ee - et + 86400) % 86400) / 60)
  // cap to sensible
  shift.start_enroll_min = Math.min(180, Math.max(0, offStart || 30))
  shift.end_enroll_min = Math.min(180, Math.max(0, offEnd || 30))
}

const shift_action = async () => { if (shift.id > 0) await edit_shift(); else await add_shift(); }

// styles for timeline preview (avoid complex inline template literals that break vue compiler)
const enrollStartStyle = computed(() => {
  const start = ((startSec.value - shift.start_enroll_min * 60) % 86400 + 86400) % 86400
  const leftPct = (start / 86400) * 100
  const wPct = (shift.start_enroll_min * 60 / 86400) * 100
  return { left: leftPct + '%', width: wPct + '%' }
})
const workStyle = computed(() => {
  const leftPct = (startSec.value / 86400) * 100
  const wPct = (durationMin.value * 60 / 86400) * 100
  return { left: leftPct + '%', width: wPct + '%' }
})
const enrollEndStyle = computed(() => {
  const leftPct = (endSec.value / 86400) * 100
  const wPct = (shift.end_enroll_min * 60 / 86400) * 100
  return { left: leftPct + '%', width: wPct + '%' }
})

defineExpose({ del_shift, edit_shift, fillForm })
</script>

<template>
  <div class="text-left">
    <form @submit.prevent="shift_action" class="panel-body space-y-4">

      <!-- context chips -->
      <div class="flex flex-wrap items-center gap-1.5">
        <span class="chip chip-quiet">{{ $t('shift.formChip') }}</span>
      </div>

      <!-- name -->
      <div>
        <label for="shift_name_new" class="label">{{ $t('shift.shiftName') }} <span class="text-danger">*</span></label>
        <input id="shift_name_new" v-model="shift.name" type="text" required :placeholder="$t('shift.shiftNamePh')"
          class="field" :aria-invalid="!!validator.name">
        <p v-if="validator.name" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{ validator.name }}
        </p>
        <p v-else class="mt-1 mono text-[10px] text-text-muted">{{ $t('shift.uniqueName') }}</p>
      </div>

      <!-- presets -->
      <section class="rounded-xl bg-surface-raised/60 border border-border p-3 space-y-2">
        <p class="card-label !text-primary-deeper">{{ $t('shift.quickPresets') }}</p>
        <div class="grid grid-cols-2 gap-2">
          <button v-for="p in presets" :key="p.label" type="button" @click="applyPreset(p)"
            class="text-left rounded-xl border border-border-strong bg-surface px-3 py-2.5 min-h-[56px] hover:bg-surface-raised hover:border-primary/40 transition">
            <span class="block text-[12px] font-bold text-text leading-none">{{ p.label }}</span>
            <span class="block mono text-[10px] text-text-muted mt-1.5">{{ p.sub }}</span>
          </button>
        </div>
      </section>

      <!-- clock in / out -->
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div>
          <label for="shift_start" class="label">{{ $t('shift.clockIn') }} <span class="text-danger">*</span></label>
          <input id="shift_start" v-model="shift.start_time_str" type="time" step="60" class="field mono"
            :aria-invalid="!!validator.start_time">
          <p v-if="validator.start_time" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
            validator.start_time }}</p>
        </div>
        <div>
          <label for="shift_end" class="label">{{ $t('shift.clockOut') }} <span class="text-danger">*</span></label>
          <input id="shift_end" v-model="shift.end_time_str" type="time" step="60" class="field mono"
            :aria-invalid="!!validator.end_time">
          <p v-if="validator.end_time" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
            validator.end_time }}</p>
        </div>
      </div>

      <!-- punch window -->
      <section class="rounded-xl border border-border p-3.5 space-y-3">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <p class="card-label !text-primary-deeper">{{ $t('shift.punchWindow') }}</p>
          <span class="mono text-[10px] text-text-muted">{{ $t('shift.workDuration') }} <b class="text-text">{{
              durationLabel }}</b></span>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <p class="label">{{ $t('shift.earliestIn') }} <span
                class="normal-case tracking-normal font-medium text-text-muted">- {{ $t('shift.beforeStart') }}</span>
            </p>
            <select v-model.number="shift.start_enroll_min" class="field" :aria-label="$t('shift.earliestIn')">
              <option :value="0">{{ $t('shift.presetOnTime') }}</option>
              <option :value="15">{{ $t('shift.presetMin15Before') }}</option>
              <option :value="30">{{ $t('shift.presetMin30Before') }}</option>
              <option :value="60">{{ $t('shift.presetMin60Before') }}</option>
              <option :value="90">{{ $t('shift.presetMin90Before') }}</option>
              <option :value="120">{{ $t('shift.presetHr2Before') }}</option>
            </select>
            <p v-if="validator.start_enroll" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
              validator.start_enroll }}</p>
            <p class="mt-1 mono text-[10px] text-text-muted">{{ fmtHM(startSec - shift.start_enroll_min * 60) }} - {{
              fmtHM(startSec) }}</p>
          </div>
          <div>
            <p class="label">{{ $t('shift.latestOut') }} <span
                class="normal-case tracking-normal font-medium text-text-muted">- {{ $t('shift.afterEnd') }}</span></p>
            <select v-model.number="shift.end_enroll_min" class="field" :aria-label="$t('shift.latestOut')">
              <option :value="0">{{ $t('shift.presetOnTime') }}</option>
              <option :value="15">{{ $t('shift.presetMin15After') }}</option>
              <option :value="30">{{ $t('shift.presetMin30After') }}</option>
              <option :value="60">{{ $t('shift.presetMin60After') }}</option>
              <option :value="90">{{ $t('shift.presetMin90After') }}</option>
              <option :value="120">{{ $t('shift.presetHr2After') }}</option>
            </select>
            <p v-if="validator.end_enroll" role="alert" class="mt-1 text-[11.5px] font-medium text-danger">{{
              validator.end_enroll }}</p>
            <p class="mt-1 mono text-[10px] text-text-muted">{{ fmtHM(endSec) }} - {{ fmtHM(endSec +
              shift.end_enroll_min*60) }}</p>
          </div>
        </div>

        <!-- timeline -->
        <div class="rounded-xl bg-canvas border border-border px-3 py-3">
          <p class="card-label mb-2">{{ $t('shift.timelinePreview') }}</p>
          <div class="relative h-9 rounded-full bg-surface border border-border overflow-hidden">
            <div class="absolute inset-y-2 left-2 right-2 rounded-full bg-surface-sunken"></div>
            <div class="absolute top-2 bottom-2 rounded-full bg-surface-sunken/80 border border-border-strong"
              :style="enrollStartStyle"></div>
            <div
              class="absolute top-1 bottom-1 rounded-full bg-primary-deep flex items-center justify-center overflow-hidden"
              :style="workStyle">
              <span class="mono text-[9px] font-bold text-white tracking-[0.06em] whitespace-nowrap">{{
                shift.start_time_str }} - {{ shift.end_time_str }}</span>
            </div>
            <div class="absolute top-2 bottom-2 rounded-full bg-surface-sunken/80 border border-border-strong"
              :style="enrollEndStyle"></div>
          </div>
          <div class="mt-2.5 flex flex-wrap gap-x-3 gap-y-1 mono text-[10px] text-text-muted">
            <span class="inline-flex items-center gap-1.5"><span class="w-2 h-2 rounded-full bg-primary-deep"></span> {{
              $t('shift.workedLabel') }} {{ durationLabel }}</span>
            <span class="inline-flex items-center gap-1.5"><span
                class="w-2 h-2 rounded-full bg-surface-sunken border border-border-strong"></span> {{
                  $t('shift.masukLabel') }} -{{ shift.start_enroll_min }}m</span>
            <span class="inline-flex items-center gap-1.5"><span
                class="w-2 h-2 rounded-full bg-surface-sunken border border-border-strong"></span> {{
                  $t('shift.pulangLabel') }} +{{ shift.end_enroll_min }}m</span>
          </div>
        </div>
      </section>

      <!-- actions -->
      <div class="flex flex-wrap items-center gap-2 pt-1">
        <button v-if="isEdit ? canEdit : canAdd" type="submit" :disabled="submitting"
          class="btn btn-primary flex-1 sm:flex-none sm:px-7">
          <font-awesome v-if="submitting" :icon="faRotate" class="w-3.5 h-3.5 animate-spin" />
          {{ submitting ? $t('common.saving') : (isEdit ? $t('common.saveChanges') : $t('shift.saveShift')) }}
        </button>
        <button v-if="isEdit" type="button" @click="callbackDoneEdit()" class="btn btn-secondary flex-1 sm:flex-none">{{
          $t('common.cancel') }}</button>
        <button v-else type="button" @click="shift.name = ''" class="btn btn-secondary flex-1 sm:flex-none">{{
          $t('common.reset') }}</button>
      </div>
    </form>
  </div>
</template>
