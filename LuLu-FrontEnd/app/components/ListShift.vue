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
    contents = computed(()=>{
        //let x = list.contents.filter(item => item.enroll_type == 1)
        return list.contents//x

    }),
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
    await $fetch(`${config.public.apiBase}/shift/list`,
    {
        method: "GET",
        headers: auth.confHeaders(),

    }).then(async (response) => {
        if (response.code == 200) {
            list.set(response.data);
        }
    });
};

function parseSeconds(input){
    //const d = new Date(1735689600+input); // 1735689600 = 2025/01/01 00:00:00
    let h = (input < 0) ? Math.floor((86400+input)/3600) : Math.floor(input/3600);
    const m = Math.floor((input % 3600) / 60);
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
}

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
                <tr v-for="(shift, index) in contents" class="*:py-1 *:text-xs *:text-center *:items-center *:px-2">
                    <td v-bind:class="format.stripes(index, '', 'bg-slate-50')" class="border-r border-gray-100 whitespace-nowrap">{{ shift.name}}</td>
                    <td v-bind:class="format.stripes(index, '', 'bg-slate-50')" class="border-r border-gray-100 whitespace-nowrap">{{ parseSeconds(shift.start_enroll) }} {{ (shift.prevday == 1) ? "(prevday)":"" }}</td>
                    <td v-bind:class="format.stripes(index, '', 'bg-slate-50')" class="border-r border-gray-100 whitespace-nowrap">{{ parseSeconds(shift.start_time) }} >> {{ parseSeconds(shift.end_time) }}</td>
                    <td v-bind:class="format.stripes(index, '', 'bg-slate-50')" class="border-r border-gray-100 whitespace-nowrap">{{ parseSeconds(shift.end_enroll) }} {{ (shift.nextday == 1) ? "(nextday)":"" }}</td>
                    <td v-bind:class="format.stripes(index, '', 'bg-slate-50')" class="whitespace-nowrap font-medium"><button class="hover:text-purple-700 hover:shadow-sm" v-on:click="callbackEdit(shift)">Edit</button> / <button class="hover:text-red-700 hover:shadow-sm" v-on:click="callbackDelete({id:shift.id, name:shift.name})">Delete</button></td>
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