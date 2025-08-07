<script setup>
import { useSchedule } from '~/store/schedule';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater, useViewUtil } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useSchedule(),
    check = useChecker(),
    format = useFormater(),
    view = useViewUtil(),
    collapse = ref({
      main:[],
      w1: [],
      w2: [],
      w3: [],
      w4: [],
    }),
    /*contents = computed(()=>{
        if (props.id > 0){
          let x = list.contents.filter(item => item.parrent == props.id)
          //x = $sortContents(x, list.contents.find(item => item.parrent == props.id)?.start_time)
          return x
        }else
          return list.contents;
    }),*/
    props = defineProps({
        id: { type: Number, required: false, default:0 },
    }),
    isOpen = ref([]);

async function initSchedule() {
    await $fetch(`${config.public.apiBase}/schedule/list`,
    {
        method: "GET",
        headers: auth.confHeaders(),

    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data);
        }
    });
};

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

      <div class="pl-2 underline text-xl text-start my-2 font-medium text-gray-600">
        Schedule
      </div>

      <div class="border-2 border-gray-100 px-4 py-2 h-[76lvh]">
        <div class="overflow-auto h-[74lvh]">
          <div v-if="list.contents.length > 0" class="w-full min-w-[30lvh]">
            <div v-for="(sch, index) in list.contents.filter(item => item.parrent == item.id)" class="mb-1">
              
              <button class="flex items-center" v-on:click="collapse.main[index] = !collapse.main[index]">
                <hr class="w-1 border mr-1"/> {{ sch.name }}
              </button>

              <div :class="collapse.main[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type == 1">
                <div class="overflow-hidden flex items-center">
                  <hr class="w-4 border mr-1"/> <span class="w-14">FLAT</span> [ <span class="w-9 ml-1">{{ format.sec_to_naive(sch.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sch.end) }}</span> ]
                </div>
              </div>

              <div :class="collapse.main[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type >= 7">
                <div class="overflow-hidden">
                  <button class="flex items-center" v-on:click="collapse.w1[index] = !collapse.w1[index]">
                    <hr class="w-4 border mr-1"/> WEEK 1
                  </button>
                  <div :class="collapse.w1[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
                    <div class="overflow-hidden">
                      <div class="flex items-center" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom <= 7).sort((a, b) => a.dom - b.dom)">
                        <hr class="w-8 border mr-1"/> <span class="w-8">{{ format.dow(index+1) }}</span> : [ <span class="w-9 ml-1">{{ format.sec_to_naive(sft.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sft.end) }}</span> ]
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div :class="collapse.main[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type >= 14">
                <div class="overflow-hidden">
                  <button class="flex items-center" v-on:click="collapse.w2[index] = !collapse.w2[index]">
                    <hr class="w-4 border mr-1"/> WEEK 2
                  </button>
                  <div :class="collapse.w2[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
                    <div class="overflow-hidden">
                      <div class="flex items-center" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 7 && item.dom <= 14).sort((a, b) => a.dom - b.dom)">
                        <hr class="w-8 border mr-1"/> <span class="w-8">{{ format.dow(index+1) }}</span> : [ <span class="w-9 ml-1">{{ format.sec_to_naive(sft.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sft.end) }}</span> ]
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div :class="collapse.main[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type >= 21">
                <div class="overflow-hidden">
                  <button class="flex items-center" v-on:click="collapse.w3[index] = !collapse.w3[index]">
                    <hr class="w-4 border mr-1"/> WEEK 3
                  </button>
                  <div :class="collapse.w3[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
                    <div class="overflow-hidden">
                      <div class="flex items-center" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 14 && item.dom <= 21).sort((a, b) => a.dom - b.dom)">
                        <hr class="w-8 border mr-1"/> <span class="w-8">{{ format.dow(index+1) }}</span> : [ <span class="w-9 ml-1">{{ format.sec_to_naive(sft.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sft.end) }}</span> ]
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div :class="collapse.main[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out" v-if="sch.type >= 28">
                <div class="overflow-hidden">
                  <button class="flex items-center" v-on:click="collapse.w4[index] = !collapse.w4[index]">
                    <hr class="w-4 border mr-1"/> WEEK 4
                  </button>
                  <div :class="collapse.w4[index] === true ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0'" class="grid overflow-hidden transition-all duration-300 ease-in-out">
                    <div class="overflow-hidden">
                      <div class="flex items-center" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 21 && item.dom <= 28).sort((a, b) => a.dom - b.dom)">
                        <hr class="w-8 border mr-1"/> <span class="w-8">{{ format.dow(index+1) }}</span> : [ <span class="w-9 ml-1">{{ format.sec_to_naive(sft.start) }}</span> => <span class="w-9 ml-0.5">{{ format.sec_to_naive(sft.end) }}</span> ]
                      </div>
                    </div>
                  </div>
                </div>
              </div>

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