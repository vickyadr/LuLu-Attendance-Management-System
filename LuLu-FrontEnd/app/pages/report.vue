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
    old_range = ref("");

async function getDataReport(ev){
    //const tz = new Date().getTimezoneOffset()* 60 * -1
    const start = ev.startTimestamp
    const end = ev.endTimestamp
    const new_range = start.toString() + end.toString()
    if (new_range === old_range.value)
        return;

    await $fetch(`${config.public.apiBase}/report/range/${start}/${end}`,
    {
        method: "GET",
        headers: auth.confHeaders(),
    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data)
            old_range.value = new_range
        }
    });
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
        <div class="flex justify-between items-center pr-5 pl-2">

            <SimpleDatePicker v-on:date-range-selected="getDataReport" :use-utc="true" :init-start-date="format.stamp_to_naive_date()" :init-end-date="format.stamp_to_naive_date()" />

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

        <ListReport :filter="filter"/>
    </div>
</template>