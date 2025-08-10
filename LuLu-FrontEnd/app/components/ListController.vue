<script setup>
import { useDeviceStore } from '~/store/devices';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useDeviceStore(),
    check = useChecker(),
    format = useFormater(),
    emit = defineEmits(['delete', 'edit']),
    contents = computed(()=>{
        //let x = list.contents.filter(item => item.enroll_type == 1)
        return list.contents//x

    });

async function initListController() {

    await $fetch(`${config.public.apiBase}/device/list`,
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

function parseStatus(input){
    if (check.isNull(input))
        return "Unknown";
    return check.inSwitch(input, ["Not Registered", "Online", "Offline", "Disable", "Syncronize"]);
}

onMounted(()=>{
    initListController()
});
</script>

<template>
    <label for="Search">
        <div class="flex justify-end items-center pr-5 pl-2">

        <div class="relative max-w-[48lvh]">
        <input
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
    </label>
    
    <div class="max-h-[30lvh] overflow-auto">
        <table class="w-full min-w-fit">
            
            <thead class="items-center items-stretch mb-1 sticky top-0">
                <tr class="*:py-1.5 *:bg-teal-600 *:text-xs *:font-bold *:text-slate-100 *:text-center *:items-center *:justify-center *:whitespace-nowrap *:px-4 *:border-r *:border-gray-200">   
                    <td class="rounded-tl-lg">No.</td>
                    <td>Serial Number</td>
                    <td>Name</td>
                    <td>Location</td>
                    <td>Service Handler</td>
                    <td>Status</td>
                    <td>Last Sync</td>
                    <td class="rounded-tr-lg">Action</td>
                </tr>
            </thead>
        
            <tbody v-if="contents.length > 0">
                <tr v-for="(device, index) in contents" class="*:py-1 *:text-xs *:text-center *:items-center *:justify-center *:whitespace-nowrap *:px-2 odd:bg-slate-50 *:border-gray-100">
                    <td class="border-r">{{ index + 1 }}</td>
                    <td class="border-r">{{ device.sn }}</td>
                    <td class="border-r">{{ device.name }}</td>
                    <td class="border-r">{{ device.location }}</td>
                    <td class="border-r">{{ device.handler }}</td>
                    <td class="border-r">{{ parseStatus(device.status) }}</td>
                    <td class="border-r">{{ format.stamp_to_naive(device.last_sync) }}</td>
                    <td><button class="hover:text-purple-700 hover:shadow-sm" v-on:click="callbackEdit(device.id)">Edit</button> / <button class="hover:text-red-700 hover:shadow-sm" v-on:click="callbackDelete(device.id)">Delete</button></td>
                </tr>
            </tbody>

            <tbody v-else>
                <tr>
                    <td colspan="8" class="py-0.5 bg-gray-50 text-sm text-center items-center justify-center rounded-b-lg">No controller registered</td>
                </tr>
            </tbody>
        </table>
    </div>

</template>