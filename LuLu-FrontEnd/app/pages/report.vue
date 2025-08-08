<script setup>
import { useReportStore } from '~/store/report';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater, useViewUtils } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useReportStore(),
    check = useChecker(),
    format = useFormater(),
    filter = ref(""),
    view = useViewUtils(),
    range = reactive({
        start: {
            day: null,
            month: null,
            year: null
        },
        end: {
            day: null,
            month: null,
            year: null
        },
    }),
    old_range = ref(null);

async function getDataReport(){
    if (!check.inRange(range.start.day, 1, 31))
        return;
    if (!check.inRange(range.start.month, 1, 12))
        return;
    if (range.start.year < 1970)
        return;
    
    if (!check.inRange(range.end.day, 1, 31))
        return;
    if (!check.inRange(range.end.month, 1, 12))
        return;
    if (range.end.year < 1970)
        return;

    const tz = new Date().getTimezoneOffset()* 60 * -1
    const start = format.define_timestamp(range.start.year, range.start.month, range.start.day) + tz
    const end = format.define_timestamp(range.end.year, range.end.month, range.end.day) + tz
    const new_range = start.toString() +  end.toString()
    
    if (new_range == old_range.value)
        return;

    await $fetch(`${config.public.apiBase}/report/range/${start}/${end}`,
    {
        method: "GET",
        headers: auth.confHeaders(),
    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data)
            old_range.value = start.toString() +  end.toString()
        }
    });
}

function dateKey(ev){
    if (ev.key == "ArrowLeft"){

    }
    
    if (ev.key == "ArrowRight"){
        
    }

    if (ev.key == "Enter"){
        getDataReport()
    }
}

definePageMeta({
    middleware: ["get-auth"],
    layout: 'default',
});

onMounted(()=> {

})
</script>

<template>
    <div class="h-[90lvh] border-2 rounded-xl border-purple-100 shadow-sm">

        <div class="">
            <div class="flex justify-between items-center pr-5 pl-2">
                <div class="flex justify-between text-nowrap text-gray-700">
                    <span>Period :</span>
                    <div class="flex justify-center px-2 mx-2 border-2 border-purple-100 hover:border-purple-200 rounded-lg">
                        <input id="start_day" v-model.number="range.start.day" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-7" placeholder="DD" maxlength="2"/>
                        <span>/</span>
                        <input id="start_month" v-model.number="range.start.month" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-7" placeholder="MM" maxlength="2"/>
                        <span>/</span>
                        <input id="start_year" v-model.number="range.start.year" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-10" placeholder="YYYY" maxlength="4"/>
                    </div>
                    <span>to</span> 
                    <div class="flex justify-center px-2 mx-2 border-2 border-purple-100 hover:border-purple-200 rounded-lg">
                        <input id="end_day" v-model="range.end.day" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-7" placeholder="DD" maxlength="2"/>
                        <span>/</span>
                        <input id="end_month" v-model="range.end.month" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-7" placeholder="MM" maxlength="2"/>
                        <span>/</span>
                        <input id="end_year" v-model="range.end.year" v-on:keydown="dateKey" v-on:blur="getDataReport" inputmode="numeric" class="outline-transparent scale-90 w-10" placeholder="YYYY" maxlength="4"/>
                    </div>         
                </div>

                <div class="relative max-w-[48lvh]">
                    <input
                        v-model="filter"
                        type="text"
                        id="Search"
                        placeholder="Search"
                        class="my-2 px-2 pb-1 w-full rounded-2xl border-2 border-purple-300 focus:border-purple-500 hover:border-purple-400 shadow-sd sm:text-md"
                    />
                
                    <span class="absolute inset-y-0 right-1 grid w-8 place-content-center">
                        <button
                        type="button"
                        aria-label="Submit"
                        class="rounded-full p-1.5 text-gray-700 transition-colors hover:bg-gray-100"
                        >
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="size-4"
                        >
                            <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
                            />
                        </svg>
                        </button>
                    </span>
                </div>
            </div>
        </div>

        <ListReport :filter="filter"/>
    </div>
</template>