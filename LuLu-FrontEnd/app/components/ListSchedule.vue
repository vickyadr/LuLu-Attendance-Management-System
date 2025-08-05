<script setup>
import { useSchedule } from '~/store/schedule';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useSchedule(),
    check = useChecker(),
    format = useFormater(),
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
    });

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

      <div class="border-2 border-gray-100 px-4 py-2 ">
        <div class="overflow-auto max-h-[76lvh]">
          <div v-if="list.contents.length > 0" class="w-full min-w-[37lvh]">
            <div v-for="(sch, index) in list.contents.filter(item => item.parrent == item.id)">
              
              <div class="flex">
                - {{ sch.name }}
              </div>

              <div class="flex ml-4" v-if="sch.type == 1">
                - FLAT [ {{ format.sec_to_naive(sch.start) }} => {{ format.sec_to_naive(sch.end) }} ]
              </div>

              <div v-if="sch.type >= 7">
                <div class="flex ml-4">
                  - WEEK 1
                </div>
                <div class="flex ml-8" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom <= 7).sort((a, b) => a.dom - b.dom)">
                  - {{ format.dow(index+1) }} : [ {{ format.sec_to_naive(sft.start) }} => {{ format.sec_to_naive(sft.end) }} ]
                </div>
              </div>

              <div v-if="sch.type >= 14">
                <div class="flex ml-4">
                  - WEEK 2
                </div>
                <div class="flex ml-8" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 7 && item.dom <= 14).sort((a, b) => a.dom - b.dom)">
                  - {{ format.dow(index+1) }} : [ {{ format.sec_to_naive(sft.start) }} => {{ format.sec_to_naive(sft.end) }} ]
                </div>
              </div>

              <div v-if="sch.type >= 21">
                <div class="flex ml-4">
                  - WEEK 3
                </div>
                <div class="flex ml-8" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 14 && item.dom <= 21).sort((a, b) => a.dom - b.dom)">
                  - {{ format.dow(index+1) }} : [ {{ format.sec_to_naive(sft.start) }} => {{ format.sec_to_naive(sft.end) }} ]
                </div>
              </div>

              <div v-if="sch.type == 28">
                <div class="flex ml-4">
                  - WEEK 4
                </div>
                <div class="flex ml-8" v-for="(sft, index) in list.contents.filter(item => item.parrent == sch.id && item.dom > 21 && item.dom <= 28).sort((a, b) => a.dom - b.dom)">
                  - {{ format.dow(index+1) }} : [ {{ format.sec_to_naive(sft.start) }} => {{ format.sec_to_naive(sft.end) }} ]
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