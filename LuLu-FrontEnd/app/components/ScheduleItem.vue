<script setup>
import { faEdit, faMinus, faPen, faPlus } from '@fortawesome/free-solid-svg-icons';
import { useSchedule } from '~/store/schedule';

const props = defineProps({
  sch: { type: Object, required: true },
  list: { type: Object, required: true },
  index: { type: Number, required: true },
  action: { type: String, required: false },
  collapse: { type: Object, required: true },
  format: { type: Object, required: true }
});

const emit = defineEmits(['toggle-collapse', 'handle-drop']);

const weekConfigs = [
  { minType: 7, maxType: 13, week: 1, domRange: [1, 7] },
  { minType: 14, maxType: 20, week: 2, domRange: [8, 14] },
  { minType: 21, maxType: 27, week: 3, domRange: [15, 21] },
  { minType: 28, week: 4, domRange: [22, 28] }
];

const getWeekShifts = (weekConfig) => {
  if (weekConfig.maxType) {
    return props.sch.type >= weekConfig.minType
  }
  return props.sch.type >= weekConfig.minType;
};

const handleDragEnter = (e) => {
  e.preventDefault()
  e.dataTransfer.dropEffect = 'copy';
}
</script>

<template>
  <div class="flex">
    <button class="flex items-center" @click="emit('toggle-collapse', 'main', sch.id)">
      <font-awesome :icon="faMinus" class="scale-75 text-gray-400" v-if="collapse.main[sch.id]" />
      <font-awesome :icon="faPlus" class="scale-75 text-gray-400" v-else />
      <span>{{ sch.name }}</span>
    </button>
    <div class="flex ml-2 h-5 border-l" v-if="action === 'edit'">
      <button class="flex items-center py-2">
        <font-awesome :icon="faPlus" class="h-2 hover:scale-150" />
      </button>
      <button class="flex items-center py-2">
        <font-awesome :icon="faPen" class="h-2 animate-pulse hover:scale-150" />
      </button>
    </div>
  </div>

  <div :class="collapse.main[sch.id] ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type == 1">
    <div class="overflow-hidden flex items-start ml-2">
      <svg height="20" width="20" xmlns="http://www.w3.org/2000/svg">
        <line x1="0" y1="0" x2="0" y2="10" class="stroke-gray-400 stroke-2" />
        <line x1="0" y1="10" x2="20" y2="10" class="stroke-gray-400" />
      </svg>
      <div class="flex mx-1 z-5" @dragover="(e) => handleDragEnter(e)" @drop="(e) => emit('handle-drop', e, sch.id)">
        <span class="w-16">FLAT</span> [ <span class="w-9 ml-1">{{ format.sec_to_naive(sch.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sch.end) }}</span> ]
      </div>
    </div>
  </div>

  <div v-for="weekConfig in weekConfigs" :key="weekConfig.week">
    <div v-if="getWeekShifts(weekConfig)" :class="collapse.main[sch.id] ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
      <div class="overflow-hidden ml-2">
        <button class="flex items-center" @click="emit('toggle-collapse', `w${weekConfig.week}`, sch.id)">
          <svg class="size-5" xmlns="http://www.w3.org/2000/svg">
            <line x1="0" y1="0" x2="0" y2="20" class="stroke-gray-400 stroke-2" v-if="sch.type > weekConfig.minType" />
            <line x1="0" y1="0" x2="0" y2="10" class="stroke-gray-400 stroke-2" v-else />
            <line x1="0" y1="10" x2="20" y2="10" class="stroke-gray-400" />
          </svg>
          <font-awesome :icon="faMinus" class="scale-75 text-gray-400" :class="{ 'hidden': !collapse[`w${weekConfig.week}`][sch.id] }" />
          <font-awesome :icon="faPlus" class="scale-75 text-gray-400" :class="{ 'hidden': collapse[`w${weekConfig.week}`][sch.id] }" />
          <span class="w-14">WEEK {{ weekConfig.week }}</span>
        </button>
        <div :class="collapse[`w${weekConfig.week}`][sch.id] ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
          <div class="overflow-hidden">
            <div class="flex items-center" v-for="(sft, idx) in (list || []).filter(item => item.dom >= weekConfig.domRange[0] && item.dom <= weekConfig.domRange[1]).sort((a, b) => a.dom - b.dom)" :key="idx">
              <svg class="size-5" xmlns="http://www.w3.org/2000/svg">
                <line x1="0" y1="0" x2="0" y2="20" class="stroke-gray-400 stroke-2" v-if="sch.type > weekConfig.minType" />
              </svg>
              <svg class="size-5 ml-2" xmlns="http://www.w3.org/2000/svg">
                <line x1="0" y1="0" x2="0" y2="20" class="stroke-gray-400 stroke-2" v-if="idx < 6" />
                <line x1="0" y1="0" x2="0" y2="10" class="stroke-gray-400 stroke-2" v-else />
                <line x1="0" y1="10" x2="20" y2="10" class="stroke-gray-400" />
              </svg>
              <span class="w-8 ml-1">{{ format.dow(idx + 1) }}</span> : [ <span class="w-9 ml-1">{{ format.sec_to_naive(sft.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sft.end) }}</span> ]
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>