<script setup>
import { useTransaction } from '~/store/transaction';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useTransaction(),
    last_data = ref(0),
    check = useChecker(),
    format = useFormater(),
    contents = computed(()=>{
        //let x = list.contents.filter(item => item.enroll_type == 1)
        return list.contents//x

    });

async function initLiveTransaction() {

    await $fetch(`${config.public.apiBase}/transaction/live`,
    {
        method: "GET",
        headers: auth.confHeaders(),

    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data);
            last_data.value = parseInt(response.message);
        }
    });

};

function parseEnrollType(input){
    return check.inSwitch(input, ["Unknown", "Finger", "PIN", "Card", "Face"])
}

onMounted(()=>{
    initLiveTransaction()
});
</script>

<template>
    <div class="max-h-[46lvh] overflow-auto">
        <table class="w-full min-w-fit">
            
            <thead class="items-center items-stretch mb-1 sticky top-0">
                <tr class="*:py-1.5 *:bg-teal-600 *:text-xs *:font-bold *:text-slate-100 *:text-center *:items-center *:border-gray-200 *:whitespace-nowrap">   
                    <td class="border-r w-[15%]">Date & Time</td>
                    <td class="border-r w-[35%]">Employee Name</td>
                    <td class="border-r w-[20%]">Enroll Type</td>
                    <td class="w-[30%]">Location</td>
                </tr>
            </thead>

            <tbody v-if="contents.length > 0">
                <tr v-for="(enroll, index) in contents" class="*:py-1 *:text-xs *:text-center *:items-center odd:bg-slate-50 *:border-gray-100 *:whitespace-nowrap">
                    <td class="border-r">{{ format.stamp_to_naive(enroll.date_time).replace(" ", " | ") }}</td>
                    <td class="border-r">{{ enroll.first_name }} {{ enroll.last_name }}</td>
                    <td class="border-r">{{ parseEnrollType(enroll.enroll_type) }}</td>
                    <td >{{ enroll.location }}</td>
                </tr>
            </tbody>

            <tbody v-else>
                <tr>
                    <td colspan="4" class="py-0.5 bg-gray-50 text-sm text-center items-center rounded-b-lg">No transaction received</td>
                </tr>
            </tbody>

        </table>
    </div>
</template>