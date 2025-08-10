<script setup>
import { useEmployee } from '~/store/employee';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useEmployee(),
    check = useChecker(),
    format = useFormater(),
    emit = defineEmits(['delete', 'edit']),
    contents = computed(()=>{
        //let x = list.contents.filter(item => item.enroll_type == 1)
        return list.contents//x

    });

function callbackDelete(sn) {
    emit('delete', sn)
}

function callbackEdit(sn) {
    emit('edit', sn)
}

async function initEmployee() {

    await $fetch(`${config.public.apiBase}/employee/list`,
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
    initEmployee()
});
</script>

<template>
    <div class="max-h-[46lvh] overflow-auto">
        <table class="w-full min-w-fit">
            
            <thead class="items-center items-stretch mb-1 sticky top-0">
                <tr class="*:py-1.5 *:bg-teal-600 *:text-xs *:font-bold *:text-slate-100 *:text-center *:items-center *:whitespace-nowrap *:px-4 *:border-gray-200">   
                    <td class="border-r">No.</td>
                    <td class="border-r">First Name</td>
                    <td class="border-r">Last Name</td>
                    <td class="border-r">Hired Date</td>
                    <td class="border-r">Departement</td>
                    <td class="border-r">Address</td>
                    <td class="border-r">Status</td>
                    <td>Action</td>
                </tr>
            </thead>

            <tbody v-if="contents.length > 0">
                <tr v-for="(data, index) in contents" class="*:py-1 *:text-xs *:text-center *:items-center *:px-2 odd:bg-slate-50 *:border-gray-100">
                    <td class="border-r whitespace-nowrap">{{ index+1 }}</td>
                    <td class="border-r whitespace-nowrap">{{ data.first_name }}</td>
                    <td class="border-r whitespace-nowrap">{{ data.last_name }}</td>
                    <td class="border-r whitespace-nowrap">{{ data.hired }}</td>
                    <td class="border-r whitespace-nowrap">{{ data.departement }}</td>
                    <td class="border-r">{{ data.address }}</td>
                    <td class="border-r whitespace-nowrap">{{ data.status }}</td>
                    <td class="whitespace-nowrap"><button class="hover:text-purple-700 hover:shadow-sm" v-on:click="callbackEdit(data.id)">Edit</button> / <button class="hover:text-red-700 hover:shadow-sm" v-on:click="callbackDelete(data.id)">Delete</button></td>
                </tr>
            </tbody>

            <tbody v-else>
                <tr>
                    <td colspan="8" class="py-0.5 bg-gray-50 text-sm text-center items-center rounded-b-lg">No employee registered</td>
                </tr>
            </tbody>

        </table>
    </div>
</template>