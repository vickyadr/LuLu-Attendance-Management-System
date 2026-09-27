<script setup>
import { useSchedule } from '~/store/schedule';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater} from '#imports';
import { faAdd, faRefresh, faSave, faTrash, faMagnifyingGlass } from '@fortawesome/free-solid-svg-icons';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY, isOffDay } from '~/utils/labels';
import ScheduleItem from './ScheduleItem.vue';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useSchedule(),
    shift = useShiftStore(),
    check = useChecker(),
    format = useFormater(),
    emit = defineEmits(['notif']),
    dragOverId = ref(null),
    dirty = reactive(new Set()),
    saving = ref(false),
    search = ref(''),
    temp = ref([]);

const contents = computed(() => list.contents)
const filteredSchedules = computed(() => {
    const q = search.value.trim().toLowerCase()
    let parents = contents.value.filter(item => item.parrent === item.id)
    if (q) parents = parents.filter(p => (p.name||'').toLowerCase().includes(q) || String(p.id).includes(q))
    return parents
})

const hasDirty = computed(() => dirty.size > 0)
const dirtyCount = computed(() => dirty.size)

async function initSchedule() {
    try {
        const response = await $fetch(`${config.public.apiBase}/schedule/list`, { method: "GET", headers: auth.confHeaders() });
        if (response.code == 200) {
            list.set(response.data);
            temp.value = JSON.parse(JSON.stringify(response.data));
            dirty.clear()
        }
    } catch(e){ console.error(e) }
};

function refreshSchedule(){
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
// default expanded main
watchEffect(() => {
  for (const p of filteredSchedules.value) {
    if (collapse.value.main[p.id] === undefined) collapse.value.main[p.id] = true
    for (let w=1; w<=4; w++) if (collapse.value[`w${w}`][p.id] === undefined) collapse.value[`w${w}`][p.id]= true
  }
})

function onDragOver(e, id){
  e.preventDefault()
  dragOverId.value = id
  e.dataTransfer.dropEffect = 'copy'
}
function onDragLeave(){ dragOverId.value = null }

async function handleDrop(e, targetId){
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

function handlePickShift(targetId, newShiftId){
  const idx = list.contents.findIndex(x => x.id === targetId);
  if (idx===-1) return
  const row = { ...list.contents[idx] }
  const sid = Number(newShiftId)
  row.schedule_shift_id = sid
  const sh = shift.get(sid)
  if (sh) { row.shift_name = sh.name; row.start = sh.start_time; row.end = sh.end_time }
  else if (sid===0) { row.shift_name = OFF_DAY; row.start = 0; row.end = 0 }
  list.contents[idx] = row
  dirty.add(targetId)
}

async function saveChanges(){
  if (dirty.size===0) return
  saving.value = true
  let ok=0, fail=0
  for (const id of Array.from(dirty)) {
    const row = list.contents.find(x=> x.id===id)
    if (!row) { dirty.delete(id); continue }
    try {
      const res = await $fetch(`${config.public.apiBase}/schedule/edit`, {
        method: 'POST',
        body: { id, shift: [ Number(row.schedule_shift_id) ] },
        headers: auth.confHeaders(),
      })
      if (res.code===200 || res.message?.toLowerCase().includes('diperbarui')) { ok++; dirty.delete(id) }
      else fail++
    } catch { fail++ }
  }
  saving.value = false
  // reload to sync
  await initSchedule()
  emit('notif', { title: 'Save Schedule', message: fail? `${ok} saved, ${fail} failed` : `${ok} schedules updated` })
}

async function deleteSchedule(parrentId, name){
  if (!confirm(`Delete schedule "${name}"?`)) return
  try {
    const res = await $fetch(`${config.public.apiBase}/schedule/delete/${parrentId}`, { method:'GET', headers: auth.confHeaders() })
    if (res.code===200) {
      await initSchedule()
      emit('notif', { title:'Delete Schedule', message: res.message || 'Schedule deleted' })
    } else emit('notif', { title:'Gagal', message: res.message })
  } catch(e){ emit('notif', { title:'Error', message: e?.data?.message || e.message }) }
}

onMounted(()=> initSchedule());

defineExpose({ refresh: initSchedule })
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header search -->
    <div class="px-3 py-2.5 border-b border-emerald-100 bg-white flex items-center gap-2">
      <div class="relative flex-1">
        <input v-model="search" type="text" placeholder="Search schedules (name / ID)..." class="w-full pl-8 pr-3 py-2 rounded-xl border-2 border-emerald-100 bg-emerald-50/30 focus:bg-white focus:border-emerald-300 focus:ring-4 focus:ring-emerald-100 outline-none text-sm text-green-900 placeholder:text-green-900/40 transition" />
        <FontAwesome :icon="faMagnifyingGlass" class="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-emerald-400" />
        <button v-if="search" @click="search=''" class="absolute right-2 top-1/2 -translate-y-1/2 w-6 h-6 rounded-full bg-white border border-emerald-100 text-emerald-600 text-xs">×</button>
      </div>
      <button @click="refreshSchedule()" title="Reload" class="w-9 h-9 rounded-xl bg-white border-2 border-emerald-100 text-emerald-600 hover:bg-emerald-50 hover:border-emerald-200 flex items-center justify-center transition"><FontAwesome :icon="faRefresh" class="w-3.5 h-3.5" :class="saving?'animate-spin':''" /></button>
    </div>

    <!-- Toolbar -->
    <div class="px-3 py-2.5 flex items-center justify-between bg-gradient-to-r from-emerald-50/40 to-white border-b border-emerald-100">
      <div class="flex items-center gap-2">
        <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700">SCHEDULE LIST</span>
        <span class="chip chip-emerald mono !text-[9px]">{{ filteredSchedules.length }} schedules</span>
        <span v-if="hasDirty" class="chip bg-amber-500 text-white mono !text-[9px] animate-pulse">{{ dirtyCount }} unsaved</span>
      </div>
      <button @click="saveChanges" :disabled="!hasDirty || saving" class="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs font-bold mono tracking-wide transition" :class="hasDirty ? 'bg-emerald-600 text-white hover:bg-emerald-700 shadow-sm' : 'bg-slate-100 text-slate-400 cursor-not-allowed'">
        <FontAwesome :icon="faSave" class="w-3 h-3" /> {{ saving ? 'Menyimpan...' : 'Save changes' }}
      </button>
    </div>

    <!-- List -->
    <div class="flex-1 overflow-auto px-2 py-2 space-y-2 bg-emerald-50/20 min-h-[320px]">
      <div v-if="filteredSchedules.length>0" class="space-y-2">
        <div v-for="sch in filteredSchedules" :key="sch.id" class="bg-white rounded-2xl border border-emerald-100 shadow-sm overflow-hidden">
          <!-- parent header -->
          <div class="px-3 py-2.5 flex items-center gap-2 bg-gradient-to-r from-white to-emerald-50/30">
            <button @click="toggleCollapse('main', sch.id)" class="flex items-center gap-2 hover:text-emerald-700 flex-1 text-left">
              <span class="w-6 h-6 rounded-lg border flex items-center justify-center text-[10px]" :class="collapse.main[sch.id] ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white text-emerald-600 border-emerald-200'">{{ collapse.main[sch.id] ? '−' : '+' }}</span>
              <span class="font-bold text-sm text-green-900 truncate">{{ sch.name }}</span>
              <span class="chip mono !text-[9px]" :class="isOffDay(sch.shift_name) ? 'bg-slate-100 border-slate-200 text-slate-600 border' : 'chip-emerald'">{{ sch.type===1?'DAILY': sch.type===7?'WEEKLY': sch.type===14?'2 WEEKS': sch.type===21?'3 WEEKS':'MONTHLY' }}</span>
              <span class="mono text-[10px] text-emerald-700/40 hidden sm:inline">ID #{{ sch.parrent }} • {{ (contents.filter(x=>x.parrent===sch.parrent).length) }} days</span>
            </button>
            <button @click="deleteSchedule(sch.parrent, sch.name)" class="w-7 h-7 rounded-lg bg-white border border-red-100 text-red-500 hover:bg-red-50 hover:border-red-200 flex items-center justify-center" title="Delete schedule"><FontAwesome :icon="faTrash" class="w-3 h-3" /></button>
          </div>

          <!-- body -->
          <div v-show="collapse.main[sch.id]" class="border-t border-emerald-100">
            <ScheduleItem
              :sch="sch"
              :list="contents.filter(item => item.parrent === sch.parrent)"
              :collapse="collapse"
              :format="format"
              :drag-over-id="dragOverId"
              @toggle-collapse="toggleCollapse"
              @drag-over="(e,id)=>onDragOver(e,id)"
              @drag-leave="onDragLeave"
              @handle-drop="handleDrop"
              @pick-shift="handlePickShift"
            />
            <div v-if="hasDirty" class="px-3 py-2 bg-amber-50 border-t border-amber-100 mono text-[10px] text-amber-700 flex items-center gap-1.5">● Unsaved changes — click <b>Save changes</b> di atas</div>
          </div>
        </div>
      </div>

      <div v-else class="py-10 text-center">
        <div class="w-12 h-12 mx-auto rounded-2xl bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600 mb-3">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
        </div>
        <p class="text-sm font-semibold text-green-900">{{ search ? 'No results' : 'No schedules yet' }}</p>
        <p class="mono text-xs text-emerald-700/50 mt-1 max-w-[28ch] mx-auto">{{ search ? 'Try a different keyword, or clear the search' : 'Create a schedule in the right panel — pick a Daily/Weekly/Monthly pattern, then fill in a shift per day' }}</p>
        <button v-if="search" @click="search=''" class="mt-3 px-4 py-1.5 rounded-full bg-emerald-600 text-white mono text-xs">Clear search</button>
      </div>
    </div>

    <div class="px-3 py-2 bg-white border-t border-emerald-100 mono text-[10px] tracking-wide text-emerald-700/40 flex items-center gap-2">
      <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 data-tick"></span> Tip: drag a shift onto any day to replace it • 0 = Day off
    </div>
  </div>
</template>
