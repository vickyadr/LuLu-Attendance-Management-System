<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift';

const
    auth = useAuthStore(),
    shift_store = useShiftStore(),
    config = useRuntimeConfig(),
    format = useFormater(),
    check = useChecker(),
    shift = reactive({
        id: 0,
        name: '',
        start_h: 0,
        end_h: 0,
        start_enroll_h: 0,
        end_enroll_h: 0,
        start_m: 0,
        end_m: 0,
        start_enroll_m: 0,
        end_enroll_m: 0,
        passday: false,
        }),
    validator = reactive({
        name:'',
        start_time:'',
        end_time:'',
        start_enroll:'',
        end_enroll:'',
        }),
    emit = defineEmits(['done-edit', 'notif']);

function callbackDoneEdit() {
    shift.id=0
    emit('done-edit')
}

function callbackNotif(data) {
    emit('notif', data)
}

function parseTime(hour = 0, min = 0){
    //const parsed = new Date(0, 0, 0, hour, min, 0, 0);
    //const def = new Date(0, 0, 0, 0, 0, 0, 0);
    //return parsed.getSeconds()-def.getSeconds();
    const h = hour * 3600;
    const m = min * 60;
    return parseInt(h + m);
}

const add_shift = async ()=> {
    await $fetch(`${config.public.apiBase}/shift/add`,
        {
            body: {
                name: shift.name,
                start_time: parseTime(shift.start_h, shift.start_m),
                end_time: parseTime(shift.end_h, shift.end_m),
                start_enroll: parseTime(shift.start_enroll_h, shift.start_enroll_m),
                end_enroll: parseTime(shift.end_enroll_h, shift.end_enroll_m),
            },
            method: "POST",
            headers: auth.confHeaders(),

        }).then(async (response) => {
            if (response.code == 200)
                shift_store.addList(response.data[0]);
            else {
                validator.end_enroll = response.data.end_enroll;
                validator.end_time = response.data.end_time;
                validator.name = response.data.name;
                validator.start_enroll = response.data.start_enroll;
                validator.start_time = response.data.start_time;
            }

            callbackNotif({title: "Add Shift", message: response.message})
        }
    );
};

async function edit_shift(){
    await $fetch(`${config.public.apiBase}/shift/edit`,
        {
            body: {
                id: shift.id,
                name: shift.name,
                start_time: parseTime(shift.start_h, shift.start_m),
                end_time: parseTime(shift.end_h, shift.end_m),
                start_enroll: parseTime(shift.start_enroll_h, shift.start_enroll_m),
                end_enroll: parseTime(shift.end_enroll_h, shift.end_enroll_m),
            },
            method: "POST",
            headers: auth.confHeaders(),

        }).then(async (response) => {
            if (response.code == 200){
                shift_store.updateList(response.data[0]);
                shift.id = 0;
                callbackDoneEdit();
            }else {
                validator.end_enroll = response.data.end_enroll;
                validator.end_time = response.data.end_time;
                validator.name = response.data.name;
                validator.start_enroll = response.data.start_enroll;
                validator.start_time = response.data.start_time;
            }
            
            callbackNotif({title: "Edit Shift", message: response.message})
        }
    );
};

async function del_shift(id){
   await $fetch(`${config.public.apiBase}/shift/delete/${id}`, 
   {
        method: "GET",
        headers: auth.confHeaders(),
   }).then(async (response) => {
        if (response.code == 200){
            shift_store.removeList(id)
        }else {

        }
        callbackNotif({title: "Delete Shift", message: response.message})
   });
}

function fillForm(data){
    shift.id = data.id
    shift.name = data.name

    shift.start_h = format.sec_to_hours(data.start_time)
    shift.start_m = format.sec_to_minutes(data.start_time)
    shift.end_h = format.sec_to_hours(data.end_time)
    shift.end_m = format.sec_to_minutes(data.end_time)

    shift.start_enroll_h = format.sec_to_hours(data.start_time - data.start_enroll)
    shift.start_enroll_m = format.sec_to_minutes(data.start_time - data.start_enroll)
    shift.end_enroll_h = format.sec_to_hours(data.end_enroll - data.end_time)
    shift.end_enroll_m = format.sec_to_minutes(data.end_enroll - data.end_time)
}

const shift_action = async () =>{
    if (shift.id > 0)
        edit_shift();
    else
        add_shift();
}

defineExpose({
    del_shift,
    edit_shift,
    fillForm,
})
</script>

<template>
    <form @submit.prevent="shift_action">

        <div class="grid grid-cols-4 gap-x-8 gap-y-6 mt-5 mx-6">
        
            <div class="col-span-4 flex">
                <div class="relative">
                    <input v-model="shift.name" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" type="text" required="" placeholder=" " id="shift_name">
                    <label for="shift_name" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Shift name : </label>
                </div>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium">{{ validator.name }}</span>
                </p>
            </div>

            <div class="relative">
                <select v-model="shift.start_h" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="start_h">
                    <option value="0">00</option>
                    <option value="1">01</option>
                    <option value="2">02</option>
                    <option value="3">03</option>
                    <option value="4">04</option>
                    <option value="5">05</option>
                    <option value="6">06</option>
                    <option value="7">07</option>
                    <option value="8">08</option>
                    <option value="9">09</option>
                    <option value="10">10</option>
                    <option value="11">11</option>
                    <option value="12">12</option>
                    <option value="13">13</option>
                    <option value="14">14</option>
                    <option value="15">15</option>
                    <option value="16">16</option>
                    <option value="17">17</option>
                    <option value="18">18</option>
                    <option value="19">19</option>
                    <option value="20">20</option>
                    <option value="21">21</option>
                    <option value="22">22</option>
                    <option value="23">23</option>
                    <option value="24">24</option>
                </select>
                <label for="start_h" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2 text-nowrap">Start shift : </label>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium">{{ validator.start_time }}</span>
                </p>                        
            </div>

            <div class="relative">
                <select v-model="shift.start_m" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="start_m">
                    <option value="0">00</option>
                    <option value="5">05</option>
                    <option value="10">10</option>
                    <option value="15">15</option>
                    <option value="20">20</option>
                    <option value="25">25</option>
                    <option value="30">30</option>
                    <option value="35">35</option>
                    <option value="40">40</option>
                    <option value="45">45</option>
                    <option value="50">50</option>
                    <option value="55">55</option>
                </select>
            </div>

            <div class="relative">
                <select v-model="shift.end_h" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="end_h">
                    <option value="0">00</option>
                    <option value="1">01</option>
                    <option value="2">02</option>
                    <option value="3">03</option>
                    <option value="4">04</option>
                    <option value="5">05</option>
                    <option value="6">06</option>
                    <option value="7">07</option>
                    <option value="8">08</option>
                    <option value="9">09</option>
                    <option value="10">10</option>
                    <option value="11">11</option>
                    <option value="12">12</option>
                    <option value="13">13</option>
                    <option value="14">14</option>
                    <option value="15">15</option>
                    <option value="16">16</option>
                    <option value="17">17</option>
                    <option value="18">18</option>
                    <option value="19">19</option>
                    <option value="20">20</option>
                    <option value="21">21</option>
                    <option value="22">22</option>
                    <option value="23">23</option>
                    <option value="24">24</option>
                </select>
                <label for="end_h" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2 text-nowrap">End shift : </label>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium">{{ validator.end_time }}</span>
                </p>                        
            </div>

            <div class="relative">
                <select v-model="shift.end_m" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="end_m">
                    <option value="0">00</option>
                    <option value="5">05</option>
                    <option value="10">10</option>
                    <option value="15">15</option>
                    <option value="20">20</option>
                    <option value="25">25</option>
                    <option value="30">30</option>
                    <option value="35">35</option>
                    <option value="40">40</option>
                    <option value="45">45</option>
                    <option value="50">50</option>
                    <option value="55">55</option>
                </select>
            </div>

            <div class="relative">
                <select v-model="shift.start_enroll_h" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="start_enroll_h">
                    <option value="0">-0</option>
                    <option value="1">-1</option>
                    <option value="2">-2</option>
                    <option value="3">-3</option>
                    <option value="4">-4</option>
                    <option value="5">-5</option>
                    <option value="6">-6</option>
                    <option value="7">-7</option>
                    <option value="8">-8</option>
                    <option value="9">-9</option>
                    <option value="10">-10</option>
                    <option value="11">-11</option>
                    <option value="12">-12</option>
                    <option value="13">-13</option>
                    <option value="14">-14</option>
                    <option value="15">-15</option>
                </select>
                <label for="start_enroll_h" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2 text-nowrap">Start enroll : </label>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium">{{ validator.start_enroll }}</span>
                </p>
            </div>

            <div class="relative">
                <select v-model="shift.start_enroll_m" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="start_enroll_m">
                    <option value="0">00</option>
                    <option value="5">05</option>
                    <option value="10">10</option>
                    <option value="15">15</option>
                    <option value="20">20</option>
                    <option value="25">25</option>
                    <option value="30">30</option>
                    <option value="35">35</option>
                    <option value="40">40</option>
                    <option value="45">45</option>
                    <option value="50">50</option>
                    <option value="55">55</option>
                </select>
            </div>
            
            <div class="relative">
                <select v-model="shift.end_enroll_h" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="end_enroll_h">
                    <option value="0">+0</option>
                    <option value="1">+1</option>
                    <option value="2">+2</option>
                    <option value="3">+3</option>
                    <option value="4">+4</option>
                    <option value="5">+5</option>
                    <option value="6">+6</option>
                    <option value="7">+7</option>
                    <option value="8">+8</option>
                    <option value="9">+9</option>
                    <option value="10">+10</option>
                    <option value="11">+11</option>
                    <option value="12">+12</option>
                    <option value="13">+13</option>
                    <option value="14">+14</option>
                    <option value="15">+15</option>
                </select>
                <label for="end_enroll_h" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2 text-nowrap">End enroll : </label>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium">{{ validator.end_enroll }}</span>
                </p>
            </div>

            <div class="relative">
                <select v-model="shift.end_enroll_m" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="end_enroll_m">
                    <option value="0">00</option>
                    <option value="5">05</option>
                    <option value="10">10</option>
                    <option value="15">15</option>
                    <option value="20">20</option>
                    <option value="25">25</option>
                    <option value="30">30</option>
                    <option value="35">35</option>
                    <option value="40">40</option>
                    <option value="45">45</option>
                    <option value="50">50</option>
                    <option value="55">55</option>
                </select>
            </div>

            <div class="flex relative col-span-4 justify-start">
                <button type="submit" class="rounded bg-blue-600 px-10 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700">
                    Save
                </button>
                <button type="button" class="ml-4 rounded bg-gray-100 px-10 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-200" v-on:click="callbackDoneEdit()" v-show="(shift.id > 0)">
                    Cancel
                </button>
            </div>
            
        </div>
    </form>
</template>