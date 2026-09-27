<script setup>
import { faMinus, faPlus } from '@fortawesome/free-solid-svg-icons';
import { useShiftStore } from '~/store/shift'
import { OFF_DAY_LABEL, isOffDay } from '~/utils/labels';

const shiftStore = useShiftStore()
const props = defineProps({
  sch: { type: Object, required: true },
  list: { type: Object, required: true },
  collapse: { type: Object, required: true },
  format: { type: Object, required: true },
  dragOverId: { type: [Number, String], required: false, default: null }
});
const emit = defineEmits(['toggle-collapse', 'handle-drop', 'drag-over', 'drag-leave', 'pick-shift']);

const weekConfigs = [
  { minType: 7, week: 1, domRange: [1, 7], label: 'WEEK 1' },
  { minType: 14, week: 2, domRange: [8, 14], label: 'WEEK 2' },
  { minType: 21, week: 3, domRange: [15, 21], label: 'WEEK 3' },
  { minType: 28, week: 4, domRange: [22, 28], label: 'WEEK 4' }
];

function isLibur(sft) {
  return sft.schedule_shift_id === 0 || sft.schedule_shift_id === '0' || isOffDay(sft.shift_name)
}
function visibleWeeks() {
  return weekConfigs.filter(w => props.sch.type >= w.minType)
}
function dayShort(dom) {
  const dow = ((dom - 1) % 7) + 1
  return props.format.dow(dow)
}
function dayFull(dom) {
  const dow = ((dom - 1) % 7) + 1
  return props.format.dow(dow, true)
}
function onPick(id, e) {
  const v = e.target.value === '' ? null : Number(e.target.value)
  emit('pick-shift', id, v)
}
function rowClasses(sft) {
  const isOver = props.dragOverId === sft.id
  const base = "flex items-center gap-2 px-2.5 py-2 rounded-xl border-2 transition text-xs"
  if (isOver) return base + " border-emerald-400 bg-emerald-50 shadow-sm scale-[1.01]"
  if (isLibur(sft)) return base + " border-slate-200 bg-slate-50/70 hover:border-slate-300"
  return base + " border-emerald-100 bg-white hover:border-emerald-200 hover:bg-emerald-50/30"
}
</script>

<template>
  <!-- FLAT — single card -->
  <div v-if="sch.type === 1" class="px-3 py-2">
    <div class="rounded-xl border-2 p-3 flex items-center gap-3 transition"
      :class="dragOverId === sch.id ? 'border-emerald-400 bg-emerald-50 shadow-sm' : 'border-emerald-100 bg-emerald-50/30 hover:border-emerald-200'"
      @dragover="(e) => emit('drag-over', e, sch.id)" @dragleave="emit('drag-leave')"
      @drop="(e) => emit('handle-drop', e, sch.id)">
      <span
        class="hidden sm:inline-flex w-8 h-8 rounded-lg bg-emerald-600 text-white items-center justify-center mono text-[10px] font-bold">FLAT</span>
      <div class="flex-1 min-w-0">
        <p class="mono text-[10px] font-bold tracking-widest text-emerald-700">1 SHIFT FOR ALL DAYS</p>
        <div class="mt-1 flex flex-wrap items-center gap-2">
          <span v-if="isLibur(sch)"
            class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-slate-100 border border-slate-200 text-slate-600 mono text-xs"><span
              class="w-1.5 h-1.5 rounded-full bg-slate-400"></span> {{ OFF_DAY_LABEL }}</span>
          <span v-else
            class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-white border border-emerald-200 text-emerald-800 mono text-xs font-medium">
            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> {{ sch.shift_name }}
            <span class="text-emerald-700/60">•</span> {{ format.sec_to_naive(sch.start) }} → {{
              format.sec_to_naive(sch.end) }}
          </span>
          <span class="mono text-[10px] text-emerald-700/40 hidden sm:inline">drag a shift here, or pick one
            below</span>
        </div>
      </div>
      <select :value="sch.schedule_shift_id" @change="onPick(sch.id, $event)"
        class="ml-auto px-2.5 py-2 rounded-xl border-2 bg-white text-xs text-green-900 outline-none min-w-[160px]"
        :class="isLibur(sch) ? 'border-slate-200' : 'border-emerald-200 focus:border-emerald-400'">
        <option :value="0">{{ OFF_DAY_LABEL }}</option>
        <option v-for="s in shiftStore.contents" :key="s.id" :value="s.id">{{ s.name }} — {{
          format.sec_to_naive(s.start_time) }}→{{ format.sec_to_naive(s.end_time) }}</option>
      </select>
    </div>
    <p class="mt-1.5 mono text-[10px] text-emerald-700/40 px-1">Drag from “Shift List”, or pick from the dropdown</p>
  </div>

  <!-- WEEKLY / BIWEEKLY / MONTHLY -->
  <div v-for="wc in visibleWeeks()" :key="wc.week" class="border-t border-emerald-100">
    <button @click="emit('toggle-collapse', `w${wc.week}`, sch.id)"
      class="w-full flex items-center gap-2 px-3 py-2 hover:bg-emerald-50/40 transition text-left">
      <span class="w-6 h-6 rounded-lg border flex items-center justify-center text-[10px]"
        :class="collapse[`w${wc.week}`][sch.id] ? 'bg-emerald-600 text-white border-emerald-600' : 'bg-white text-emerald-600 border-emerald-200'">
        <font-awesome :icon="collapse[`w${wc.week}`][sch.id] ? faMinus : faPlus" class="w-2.5 h-2.5" />
      </span>
      <span class="mono text-xs font-bold tracking-widest text-emerald-700">{{ wc.label }}</span>
      <span
        class="mono text-[10px] tracking-widest px-2 py-0.5 rounded-full bg-white border border-emerald-100 text-emerald-700/60">{{
          wc.domRange[0] }}–{{ wc.domRange[1] }}</span>
      <span class="ml-auto mono text-[10px] text-emerald-700/30 hidden sm:inline">7 days</span>
    </button>

    <div v-show="collapse[`w${wc.week}`][sch.id]" class="px-2 pb-2">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2 gap-2">
        <div
          v-for="sft in (list || []).filter(x => x.dom >= wc.domRange[0] && x.dom <= wc.domRange[1]).sort((a, b) => a.dom - b.dom)"
          :key="sft.id" :class="rowClasses(sft)" @dragover="(e) => emit('drag-over', e, sft.id)"
          @dragleave="emit('drag-leave')" @drop="(e) => emit('handle-drop', e, sft.id)">
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <span class="w-10 text-center mono text-[10px] font-bold px-1.5 py-1 rounded-lg"
              :class="((sft.dom - 1) % 7 === 0) ? 'bg-red-500 text-white' : 'bg-emerald-600 text-white'">{{ dayShort(sft.dom)
              }}</span>
            <span class="mono text-[10px] text-emerald-700/50 hidden sm:inline">D{{ sft.dom }}</span>
            <span v-if="isLibur(sft)"
              class="inline-flex items-center gap-1 px-2 py-1 rounded-full bg-slate-100 border border-slate-200 text-slate-600 mono text-xs">{{
              OFF_DAY_LABEL }}</span>
            <span v-else class="truncate mono text-xs font-medium text-green-900">{{ sft.shift_name }} <span
                class="text-emerald-700/40">•</span> <span class="text-emerald-700">{{ format.sec_to_naive(sft.start)
                }}→{{ format.sec_to_naive(sft.end) }}</span></span>
          </div>
          <select :value="sft.schedule_shift_id" @change="onPick(sft.id, $event)"
            class="shrink-0 px-2 py-1.5 rounded-lg border bg-white text-xs text-green-900 outline-none min-w-[132px] max-w-[160px]"
            :class="isLibur(sft) ? 'border-slate-200' : 'border-emerald-200 focus:border-emerald-300'">
            <option :value="0">{{ OFF_DAY_LABEL }}</option>
            <option v-for="sh in shiftStore.contents" :key="sh.id" :value="sh.id">{{ sh.name }}</option>
          </select>
        </div>
      </div>
    </div>
  </div>
</template>
