<script setup>
import { useReportStore } from '~/store/report';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useReportStore(),
    check = useChecker(),
    format = useFormater(),
    emit = defineEmits(['delete', 'edit']),
    props = defineProps({
        filter: {type: String, required: true},
    }),
    contents = computed(()=>{
        if (check.isNull(props.filter) || check.isNull(list.contents))
            return list.contents

        return list.contents.filter(item => (item.first_name +" "+ item.last_name).toLowerCase().includes(props.filter.toLowerCase()))

    });

async function initReport() {

    await $fetch(`${config.public.apiBase}/report/today`,
    {
        method: "GET",
        headers: auth.confHeaders(),
    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data);
        }
    });

};

function callbackDelete(sn) {
    emit('delete', sn)
}

function callbackEdit(sn) {
    emit('edit', sn)
}

function shift_prevday(input){
    input.shift_end
    const sub_24 = input - 86400;
    if (sub_24 < 0)
        return sub_24
    else
        return input
}

onMounted(()=>{
    //initReport()
});
</script>

<template>
    <div class="max-h-[40lvh] overflow-auto">
        <table class="w-full min-w-fit">
            
            <thead class="items-center items-stretch sticky top-0">
                <tr class="*:py-1.5 *:bg-teal-600 *:text-xs *:font-bold *:text-slate-100 *:text-center *:items-center *:whitespace-nowrap *:px-4"> 
                    <td class="border-r w-[19%]">Name</td>
                    <td class="border-r w-[19%]">Departement</td> 
                    <td class="border-r w-[9%]">Date</td>
                    <td class="border-r w-[9%]">Weekday</td>
                    <td class="border-r w-[9%]">Shift</td> 
                    <td class="border-r w-[9%]">IN</td>
                    <td class="border-r w-[9%]">OUT</td>
                    <td class="border-r w-[9%]">Work Hour</td>
                    <td class="w-[8%]">Late</td>
                </tr>
            </thead>

            <tbody v-if="contents.length > 0">
                <tr v-for="(data, index) in contents" class="*:py-1 *:text-xs *:text-center *:items-center *: whitespace-nowrap *:px-2 odd:bg-slate-50 *:border-gray-100">
                    <td class="border-r">{{ data.first_name }} {{ data.last_name }}</td>
                    <td class="border-r">{{ data.departement }}</td>
                    <td class="border-r">{{ format.stamp_to_naive_date(data.date) }}</td>
                    <td class="border-r">{{ format.stamp_to_weekday(data.date, null, true) }}</td>
                    <td class="border-r">{{ format.sec_to_naive(data.shift_start) }} >> {{ format.sec_to_naive(data.shift_end) }}</td>
                    <td class="border-r">{{ (!check.isNull(data.start)) ? format.stamp_to_naive_time(data.start) : "-" }}</td>
                    <td class="border-r">{{ (!check.isNull(data.end)) ? format.stamp_to_naive_time(data.end) : "-" }}</td>
                    <td class="border-r text-red-600" v-if="data.working_time < data.shift_end - data.shift_start">{{ format.sec_to_naive(data.working_time) }}</td>
                    <td class="border-r" v-else>{{ format.sec_to_naive(data.working_time) }}</td>
                    <td class="text-red-600" v-if="data.late_time > 0">{{ format.sec_to_naive(data.late_time) }}</td>
                    <td class="" v-else> - </td>
                </tr>
            </tbody>

            <tbody v-else>
                <tr>
                    <td colspan="9" class="py-1 bg-gray-50 text-sm text-center items-center rounded-b-lg">No enroll found</td>
                </tr>
            </tbody>

        </table>
    </div>
</template>