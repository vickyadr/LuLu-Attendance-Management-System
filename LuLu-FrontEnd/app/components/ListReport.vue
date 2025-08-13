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
    <div class="max-h-[82lvh] overflow-auto">
        <table class="w-full min-w-fit divide-y divide-gray-200">
            <thead class="bg-gray-50 sticky top-0">
                <tr class="*:py-2 *:px-4 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    <th>
                        Employee
                    </th>
                    <th>
                        Date
                    </th>
                    <th>
                        Shift
                    </th>
                    <th>
                        Enroll
                    </th>
                    <th>
                        Tags
                    </th>
                    <th>
                        Details
                    </th>
                </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200" v-if="!check.isNull(contents)">
                <tr v-for="issue in contents" :key="`${issue.id}-${issue.date}`" class="hover:bg-gray-50 *:py-2 *:px-4 text-sm">
                    <td>
                        <div class="whitespace-nowrap font-medium text-gray-900">
                            {{ issue.first_name }} {{ issue.last_name }}
                        </div>
                        <div class="whitespace-nowrap text-gray-500">
                            {{ issue.departement }}
                        </div>
                    </td>
                    <td>
                        <span class="whitespace-nowrap text-gray-900">
                            {{ format.stamp_to_naive_date(issue.date) }}
                        </span>
                    </td>
                    <td>
                        <div class="flex text-gray-900 whitespace-nowrap">
                            <span class="w-8">Start</span> : {{ format.sec_to_naive(issue.shift_start) }}
                        </div>
                        <div class="flex text-gray-900 whitespace-nowrap">
                            <span class="w-8">End</span> : {{ format.sec_to_naive(issue.shift_end) }}
                        </div>
                    </td>
                    <td>
                        <div class="flex text-gray-900 whitespace-nowrap">
                            <span class="w-8">In</span> : {{ (!check.isNull(issue.start)) ? format.stamp_to_naive_time(issue.start, false) : "-" }}
                        </div>
                        <div class="flex text-gray-900 whitespace-nowrap">
                            <span class="w-8">Out</span> : {{ (!check.isNull(issue.end)) ? format.stamp_to_naive_time(issue.end) : "-" }}
                        </div>
                    </td>
                    <td class="flex-wrap">
                        <span
                            class="whitespace-nowrap px-2 mr-2 inline-flex shadow-md font-semibold rounded-full text-xs bg-red-100 text-red-800"
                            v-if="issue.late_time > 0"
                            >Late Arrival</span>
                        <span
                            class="whitespace-nowrap px-2 mr-2 inline-flex shadow-md font-semibold rounded-full text-xs bg-yellow-100 text-yellow-800"
                            v-if="issue.working_time < issue.shift_end - issue.shift_start"
                            >Early Leave</span>
                        <span
                            class="whitespace-nowrap px-2 mr-2 inline-flex shadow-md font-semibold rounded-full text-xs bg-blue-100 text-blue-800"
                            v-if="check.isNull(issue.start) && check.isNull(issue.end)"
                            >Absence</span>
                        <span
                            class="whitespace-nowrap px-2 mr-2 inline-flex shadow-md font-semibold rounded-full text-xs bg-purple-100 text-purple-800"
                            v-if="issue.working_time > issue.shift_end - issue.shift_start"
                            >Overtime</span>
                    </td>
                    <td>
                        <div class="flex text-red-600 whitespace-nowrap" v-if="issue.late_time > 0">
                            <span class="w-14">Late</span> : {{ format.sec_to_naive(issue.late_time) }}
                        </div>
                        <div class="flex text-red-600 whitespace-nowrap" v-if="issue.working_time < issue.shift_end - issue.shift_start">
                            <span class="w-14">Working</span> : {{ format.sec_to_naive(issue.working_time) }}
                        </div>
                        <div class="flex text-gray-900 whitespace-nowrap" v-else>
                            <span class="w-14">Working</span> : {{ format.sec_to_naive(issue.working_time) }}
                        </div>
                    </td>
                </tr>
            </tbody>
            <tbody v-else>
                <tr>
                    <td colspan="6" class="px-6 py-2 whitespace-nowrap text-center">No enroll found</td>
                </tr>
            </tbody>
        </table>
    </div>
</template>