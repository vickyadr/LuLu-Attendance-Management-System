<script setup>
import { useSchedule } from '~/store/schedule';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater} from '#imports';
import { faAdd, faEdit, faMinus, faPen, faPlus, faRefresh, faSave } from '@fortawesome/free-solid-svg-icons';
import { useShiftStore } from '~/store/shift';
import ScheduleItem from './ScheduleItem.vue';

var temp = {value:[]};
const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useSchedule(),
    shift = useShiftStore(),
    check = useChecker(),
    format = useFormater(),
    action = ref(null),
    dragState = ref(null),
    collapse = ref({
      main: [],
      w1: [],
      w2: [],
      w3: [],
      w4: [],
    }),
    props = defineProps({
        id: { type: Number, required: false, default: 0 },
    }),
    contents = computed(() => list.contents),
    filteredSchedules = computed(() => contents.value.filter(item => item.parrent === item.id));

async function initSchedule() {
    await $fetch(`${config.public.apiBase}/schedule/list`,
    {
        method: "GET",
        headers: auth.confHeaders(),

    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data);
            temp.value = JSON.parse(JSON.stringify(response.data));
        }
    });
};

function addSchedule(){
    const sortId = JSON.parse(JSON.stringify(contents.value)).sort((a,b) => b.id - a.id)
    const lastId = sortId.at(0).id+1;
    list.contents.push({
      "id": lastId,
      "name": `Schedule-${lastId}`,
      "schedule_shift_id": 0,
      "shift_name": "",
      "start": 0,
      "end": 0,
      "type": 1,
      "dom": 1,
      "parrent": lastId
    })
}

const handleDragEnter = (e) => {
  e.preventDefault()
  e.dataTransfer.dropEffect = 'copy';
}

const handleDrop = (e, id) =>{
  e.preventDefault()
  const data = e.dataTransfer.getData('shift')
  const js = JSON.parse(data)

  const getShift = shift.get(js.id);
  let index = list.contents.findIndex(x => x.id === id);
  let getSch = list.contents[index];

  getSch.schedule_shift_id = getShift.id;
  getSch.start = getShift.start_time;
  getSch.end = getShift.end_time;
  getSch.change = true;
  list.contents[index] = getSch;
}

const refreshSchedule = () =>{
  list.contents = JSON.parse(JSON.stringify(temp.value))
}

function toggleCollapse(type, index) {
  collapse.value[type][index] = !collapse.value[type][index];
}

onMounted(()=>{
  if(props.id == 0)
    initSchedule();
});
</script>

<template>

  <div class="flex justify-end items-center pr-3 pl-2 border-b-2 border-gray-200 shadow-sm">

    <div class="relative max-w-[42lvh]">
      <input
        type="text"
        id="Search"
        placeholder="Search"
        class="my-2 px-2 pb-1 w-full rounded-2xl border-2 border-purple-100 hover:border-purple-200 focus:border-purple-300 pe-10 shadow-sd sm:text-md"/>
  
      <span class="absolute inset-y-0 right-1 grid w-8 place-content-center">
        <button
          type="button"
          aria-label="Submit"
          class="rounded-full p-1.5 text-gray-700 transition-colors hover:bg-gray-100">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="1.5"
            stroke="currentColor"
            class="size-4">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"/>
          </svg>

        </button>
      </span>
    </div>
  </div>

  <div class="px-2.5">

      <div class="flex justify-between pl-2 underline text-xl text-start my-2 font-medium text-gray-600">
        <div><span class="shadow-md mr-2">Schedule</span></div>
        <div>
          <button class="mx-2 p-0.5 text-yellow-500 shadow-md rounded-lg">
            <FontAwesome @click="refreshSchedule()" :icon="faRefresh" class="hover:animate-spin"/>
          </button>
          <button @click="addSchedule()" class="mx-2 p-0.5 text-purple-700 shadow-md rounded-lg">
            <FontAwesome :icon="faAdd" class="hover:animate-pulse"/>
          </button>
          <button disabled class="disabled:text-gray-400 mx-2 p-0.5 text-teal-500 shadow-md rounded-lg">
            <FontAwesome :icon="faSave" class="hover:animate-pulse"/>
          </button>
        </div>
      </div>

      <div class="border-2 border-gray-100 px-4 py-2 h-[76lvh]">
        <div class="overflow-auto h-[74lvh]">
          <div v-if="contents.length > 0" class="w-full min-w-[28lvh]">
            <div v-for="(sch, index) in filteredSchedules" :key="sch.id" class="mb-1">
              <ScheduleItem
                :sch="sch"
                :list="contents.filter(item => item.parrent === sch.parrent)"
                :index="index"
                :action="action"
                :collapse="collapse"
                :format="format"
                @toggle-collapse="toggleCollapse"
                @handle-drop="handleDrop"
              />
            </div>
          </div>

          <div v-else>
            <div class="py-0.5 text-sm text-gray-600 text-start text-lg col-span-4 rounded-b-lg">
              <span>&gt; </span>
              <span class="underline decoration-2 decoration-slate-300 decoration-dotted">No schedule have been made</span>
            </div>
          </div>
        </div>
      </div>

  </div>

</template>