<script setup>
import { useSchedule } from '~/store/schedule';
import { useAuthStore } from '~/store/auth';
import { useNotifyStore } from '~/store/notify';
import { useChecker, useFormater } from '#imports';
import { faRefresh, faTrash, faMagnifyingGlass, faInbox, faXmark, faMinus, faPlus, faFloppyDisk } from '@fortawesome/free-solid-svg-icons';
import { useShiftStore } from '~/store/shift'
import { SMData } from '~/components/SimpleModal.vue'
import { OFF_DAY, isOffDay } from '~/utils/labels';
import ScheduleItem from './ScheduleItem.vue';

const { t } = useI18n()
const sm_data = new SMData()

const
  auth = useAuthStore(),
  notify = useNotifyStore(),
  config = useRuntimeConfig(),
  list = useSchedule(),
  shift = useShiftStore(),
  check = useChecker(),
  format = useFormater(),
  dragOverId = ref(null),
  dirty = reactive(new Set()),
  saving = ref(false),
  search = ref(''),
  temp = ref([]);

// /schedule/* is gated on the 'shift' page server-side, so the UI must agree.
// Declared after `auth` exists: a computed referencing `auth` above its
// declaration throws on first access, not at definition time.
const canEdit = computed(() => auth.hasActionAccess('shift', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('shift', 'can_delete'))

const contents = computed(() => list.contents)
const filteredSchedules = computed(() => {
  const q = search.value.trim().toLowerCase()
  let parents = contents.value.filter(item => item.parent === item.id)
  if (q) parents = parents.filter(p => (p.name || '').toLowerCase().includes(q) || String(p.id).includes(q))
  return parents
})

const hasDirty = computed(() => dirty.size > 0)
const dirtyCount = computed(() => dirty.size)

async function initSchedule() {
  try {
    const response = await apiFetch(`${config.public.apiBase}/schedule/list`, { method: "GET", headers: auth.confHeaders() });
    if (response.code == 200) {
      list.set(response.data);
      temp.value = JSON.parse(JSON.stringify(response.data));
      dirty.clear()
    }
  } catch (e) { console.error(e) }
};

function refreshSchedule() {
  if (temp.value.length) {
    list.contents = JSON.parse(JSON.stringify(temp.value))
    dirty.clear()
  } else initSchedule()
}

const collapse = ref({ main: [], w1: [], w2: [], w3: [], w4: [] })
function toggleCollapse(type, index) {
  if (collapse.value[type][index] === undefined) collapse.value[type][index] = false
  collapse.value[type][index] = !collapse.value[type][index];
}
// Default folded: both the parent schedules and every week stay collapsed
// until clicked. Only rows the user explicitly toggles are tracked, so the
// undefined check below must keep assigning a default rather than assume one.
watchEffect(() => {
  for (const p of filteredSchedules.value) {
    if (collapse.value.main[p.id] === undefined) collapse.value.main[p.id] = false
    for (let w = 1; w <= 4; w++) if (collapse.value[`w${w}`][p.id] === undefined) collapse.value[`w${w}`][p.id] = false
  }
})

function onDragOver(e, id) {
  e.preventDefault()
  dragOverId.value = id
  e.dataTransfer.dropEffect = 'copy'
}
function onDragLeave() { dragOverId.value = null }

async function handleDrop(e, targetId) {
  e.preventDefault()
  dragOverId.value = null
  let raw
  try { raw = e.dataTransfer.getData('shift') } catch { return }
  if (!raw) return
  let js
  try { js = JSON.parse(raw) } catch { return }
  const getShift = shift.get(js.id);
  if (!getShift) return
  const idx = list.contents.findIndex(x => x.id === targetId);
  if (idx === -1) return
  const row = { ...list.contents[idx] }
  row.schedule_shift_id = getShift.id;
  row.shift_name = getShift.name;
  row.start = getShift.start_time;
  row.end = getShift.end_time;
  // mark dirty for save
  list.contents[idx] = row
  dirty.add(targetId)
}

function handlePickShift(targetId, newShiftId) {
  const idx = list.contents.findIndex(x => x.id === targetId);
  if (idx === -1) return
  const row = { ...list.contents[idx] }
  const sid = Number(newShiftId)
  row.schedule_shift_id = sid
  const sh = shift.get(sid)
  if (sh) { row.shift_name = sh.name; row.start = sh.start_time; row.end = sh.end_time }
  else if (sid === 0) { row.shift_name = OFF_DAY; row.start = 0; row.end = 0 }
  list.contents[idx] = row
  dirty.add(targetId)
}

async function saveChanges() {
  if (dirty.size === 0) return
  saving.value = true
  let ok = 0, fail = 0
  for (const id of Array.from(dirty)) {
    const row = list.contents.find(x => x.id === id)
    if (!row) { dirty.delete(id); continue }
    try {
      const res = await apiFetch(`${config.public.apiBase}/schedule/edit`, {
        method: 'POST',
        body: { id, shift: [Number(row.schedule_shift_id)] },
        headers: auth.confHeaders(),
      })
      if (res.code === 200 || res.message?.toLowerCase().includes('diperbarui')) { ok++; dirty.delete(id) }
      else fail++
    } catch { fail++ }
  }
  saving.value = false
  // reload to sync
  await initSchedule()
  notify[fail ? 'warning' : 'success'](t('schedule.saveSchedule'), fail ? t('schedule.saveSummaryFail', { ok, fail }) : t('schedule.saveSummaryOk', { ok }))
}

async function deleteSchedule(parentId, name) {
  sm_data.setText(t('schedule.deleteTitle'), t('schedule.deleteConfirm', { name }))
  sm_data.setHelper("del-schedule", { parentId })
  sm_data.showOKCancel()
}
async function handleDeleteOk() {
  const helper = sm_data.getHelper("del-schedule")
  if (helper) {
    try {
      const res = await apiFetch(`${config.public.apiBase}/schedule/delete/${helper.parentId}`, { method: 'GET', headers: auth.confHeaders() })
      if (res.code === 200) {
        await initSchedule()
        notify.success(t('schedule.deleteTitle'), res.message || t('schedule.deleted'))
      } else notify.error(t('common.deleteFailed'), res.message)
    } catch (e) { notify.error(t('common.deleteFailed'), e?.data?.message || e.message) }
  }
  sm_data.clear()
}
function handleDeleteCancel() { sm_data.clear() }

onMounted(() => initSchedule());

defineExpose({ refresh: initSchedule })
</script>

<template>
  <div class="flex flex-col h-full">

    <!-- search -->
    <div class="px-3 py-2.5 border-b border-border flex items-center gap-2">
      <div class="relative flex-1">
        <font-awesome :icon="faMagnifyingGlass"
          class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
        <input v-model="search" type="text" :placeholder="$t('schedule.searchPh')" :aria-label="$t('schedule.searchPh')"
          class="field has-icon-left has-icon-right" />
        <button v-if="search" @click="search = ''"
          class="absolute right-1 top-1/2 -translate-y-1/2 w-9 h-8 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised transition"
          :aria-label="$t('common.clearSearch')">
          <font-awesome :icon="faXmark" class="w-3 h-3" />
        </button>
      </div>
      <button @click="refreshSchedule()" :title="$t('device.reloadShort')" :aria-label="$t('device.reloadShort')"
        class="w-9 h-9 rounded-lg border border-border text-text-muted hover:bg-surface-raised hover:text-primary-deeper transition grid place-content-center">
        <font-awesome :icon="faRefresh" class="w-3.5 h-3.5" :class="saving ? 'animate-spin' : ''" />
      </button>
    </div>

    <!-- toolbar -->
    <div class="px-3 py-2 flex flex-wrap items-center justify-between gap-2 border-b border-border">
      <div class="flex items-center gap-1.5 min-w-0">
        <span class="mono text-[10px] font-bold tracking-[0.14em] uppercase text-text-muted truncate">{{
          $t('schedule.listTitle') }}</span>
        <span class="chip chip-quiet shrink-0">{{ $t('schedule.countBadge', { n: filteredSchedules.length }) }}</span>
        <span v-if="hasDirty" class="chip chip-warn shrink-0">{{ $t('schedule.dirtyCount', { n: dirtyCount }) }}</span>
      </div>
      <button v-if="canEdit" @click="saveChanges" :disabled="!hasDirty || saving" class="btn btn-primary shrink-0">
        <font-awesome :icon="faFloppyDisk" class="w-3 h-3" /> {{ saving ? $t('common.saving') :
          $t('schedule.saveChanges') }}
      </button>
    </div>

    <!-- list -->
    <div class="flex-1 overflow-auto overscroll-contain px-2.5 py-2.5 space-y-2.5 bg-canvas min-h-0">

      <div v-if="filteredSchedules.length > 0" class="space-y-2.5">
        <div v-for="sch in filteredSchedules" :key="sch.id"
          class="rounded-xl border border-border bg-surface overflow-hidden">
          <!-- parent row -->
          <div class="px-3 py-1 flex items-center gap-2">
            <button @click="toggleCollapse('main', sch.id)"
              class="flex items-center gap-2 flex-1 min-w-0 min-h-[44px] text-left">
              <span class="w-6 h-6 rounded-md border grid place-content-center shrink-0 transition"
                :class="collapse.main[sch.id] ? 'bg-primary-deep border-primary-deep text-white' : 'bg-surface border-border-strong text-primary-deep'">
                <font-awesome :icon="collapse.main[sch.id] ? faMinus : faPlus" class="w-2.5 h-2.5" />
              </span>
              <span class="text-[13px] font-semibold text-text truncate">{{ sch.name }}</span>
              <span class="chip chip-quiet shrink-0">{{ sch.type === 1 ? $t('schedule.badgeDaily') :
                sch.type === 7 ? $t('schedule.badgeWeekly') : sch.type === 14 ? $t('schedule.weeks2') :
                  sch.type === 21 ? $t('schedule.weeks3') : $t('schedule.badgeMonthly') }}</span>
              <span class="mono text-[10px] text-text-muted hidden sm:inline shrink-0">#{{ sch.parent }} · {{
                $t('schedule.daysCount', { n: contents.filter(x => x.parent === sch.parent).length }) }}</span>
            </button>
            <button v-if="canDelete" @click="deleteSchedule(sch.parent, sch.name)"
              class="w-9 h-9 rounded-lg border border-border text-danger hover:bg-danger-bg transition grid place-content-center"
              :title="$t('schedule.deleteTitle')" :aria-label="$t('schedule.deleteTitle')">
              <font-awesome :icon="faTrash" class="w-3 h-3" />
            </button>
          </div>

          <!-- expanded body -->
          <div v-show="collapse.main[sch.id]" class="border-t border-border">
            <ScheduleItem :sch="sch" :list="contents.filter(item => item.parent === sch.parent)" :collapse="collapse"
              :format="format" :drag-over-id="dragOverId" @toggle-collapse="toggleCollapse"
              @drag-over="(e, id) => onDragOver(e, id)" @drag-leave="onDragLeave" @handle-drop="handleDrop"
              @pick-shift="handlePickShift" />
            <div v-if="hasDirty"
              class="px-3 py-2 bg-warn-bg border-t border-amber-200 mono text-[10px] text-amber-700 flex items-center gap-1.5">
              {{ $t('schedule.unsavedNote') }}
            </div>
          </div>
        </div>
      </div>

      <!-- empty -->
      <div v-else class="state">
        <div class="state-icon">
          <span class="hud-corner hud-corner-tl"></span>
          <span class="hud-corner hud-corner-br"></span>
          <font-awesome :icon="faInbox" class="w-5 h-5" />
        </div>
        <p class="state-title">{{ search ? $t('schedule.noResults') : $t('schedule.noneYet') }}</p>
        <p class="state-hint">{{ search ? $t('schedule.tryKeyword') : $t('schedule.createHint') }}</p>
        <button v-if="search" @click="search = ''" class="btn btn-secondary mt-1">{{ $t('common.clearSearch') }}</button>
      </div>
    </div>

  </div>
  <SimpleModal :options="sm_data.get()" @ok="handleDeleteOk" @cancel="handleDeleteCancel" v-show="sm_data.isShow()" />

</template>
