<script setup>
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';
import { useShiftStore } from '~/store/shift';

const 
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useShiftStore(),
    check = useChecker(),
    emit = defineEmits(['edit', 'delete']),
    format = useFormater(),
    contents = computed(() => list.contents),
    modals = reactive({
    isShow: false,
    title: '',
    message: '',
    });

function callbackEdit(id) {
    emit('edit', id)
}

function callbackDelete(id) {
    emit('delete', id)
}

async function getListShift() {
    try {
        const response = await $fetch(`${config.public.apiBase}/shift/list`, {
            method: "GET",
            headers: auth.confHeaders(),
        });
        
        if (response.code === 200) {
            list.set(response.data);
        }
    } catch (error) {
        console.error('Failed to fetch shift list:', error);
    }
}

const parseSeconds = (input) => {
    const totalSeconds = input < 0 ? 86400 + input : input;
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
};

const handleDragStart = (e, act) => {
    e.dataTransfer.setData('shift', JSON.stringify(act))
    e.dataTransfer.effectAllowed = 'copyNone'
};

onMounted(()=>{
    getListShift()
});
</script>

<template>
    <div class="h-[28.5lvh] overflow-auto">
        <table class="w-full min-w-fit">
            
            <thead class="items-center items-stretch sticky top-0">
                <tr class="*:py-1.5 *:bg-teal-600 *:text-xs *:font-bold *:text-slate-100 *:text-center *:items-center *:px-4">   
                    <td class="border-r border-gray-200 whitespace-nowrap">Shift name</td>
                    <td class="border-r border-gray-200 whitespace-nowrap">Start enroll</td>
                    <td class="border-r border-gray-200 whitespace-nowrap">Work time</td>
                    <td class="border-r border-gray-200 whitespace-nowrap">End enroll</td>
                    <td class="whitespace-nowrap">Action</td>
                </tr>
            </thead>

            <tbody v-if="contents.length > 0">
                <tr v-for="(shift, index) in contents" class="*:py-1 *:whitespace-nowrap *:text-xs *:text-center *:items-center *:px-2 odd:bg-slate-50">
                    <td draggable="true" v-on:dragstart="(e) => handleDragStart(e, {act:'copy', id:shift.id})" class="border-r border-gray-100">{{ shift.name}}</td>
                    <td class="border-r border-gray-100">{{ parseSeconds(shift.start_enroll) }} {{ (shift.prevday == 1) ? "(prevday)":"" }}</td>
                    <td class="border-r border-gray-100">{{ parseSeconds(shift.start_time) }} >> {{ parseSeconds(shift.end_time) }}</td>
                    <td class="border-r border-gray-100">{{ parseSeconds(shift.end_enroll) }} {{ (shift.nextday == 1) ? "(nextday)":"" }}</td>
                    <td class="whitespace-nowrap font-medium"><button class="hover:text-purple-700 hover:shadow-sm" v-on:click="callbackEdit(shift)">Edit</button> / <button class="hover:text-red-700 hover:shadow-sm" v-on:click="callbackDelete({id:shift.id, name:shift.name})">Delete</button></td>
                </tr>
            </tbody>

            <tbody v-else>
                <tr>
                    <td colspan="5" class="py-1 bg-gray-50 text-sm text-center items-center rounded-b-lg">No shift made</td>
                </tr>
            </tbody>

        </table>
    </div>

</template>