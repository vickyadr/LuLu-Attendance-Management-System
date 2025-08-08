<script setup>
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useShiftStore } from '~/store/shift';
import { useSchedule } from '~/store/schedule';

const
    auth = useAuthStore(),
    schedule = useSchedule(),
    shift = useShiftStore(),
    config = useRuntimeConfig(),
    format = useFormater(),
    check = useChecker(),
    pattern = ref([]),
    form_data = reactive({
        name:'',
        pattern:0,
        shift: [],
    });

function optionUpdate(data){
    pattern.value[data.key] = data.val
    //window.alert(data.key)
    //window.alert(data.val)
    //window.alert(pattern.value.length)
}

const add_schedule = async () =>{
    await $fetch(`${config.public.apiBase}/schedule/add`,
        {
            body: {
                name: form_data.name,
                pattern: form_data.pattern,
                shift: form_data.shift,
            },
            method: "POST",
            headers: auth.confHeaders(),

        }).then(async (response) => {
            if (response.code == 200)
                schedule.addList(response.data[0]);
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

</script>

<template>
    <div class="mt-2 mx-6">
        
        <form @submit.prevent="add_schedule">

            <div class="flex my-5">

                <div class="relative">
                    <input v-model="form_data.name" class="mt-1 text-gray-700 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" type="text" required placeholder=" " id="schedule_name">
                    <label for="schedule_name" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Schedule name : </label>
                </div>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium"></span>
                </p>

            </div>

            <div class="flex my-5">

                <div class="relative">
                    <select v-model="form_data.pattern" class="mt-1 text-gray-700 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="schedule_pattern">
                        <option value="0" default>Flat</option>
                        <option value="1">Weekly (un-implemented)</option>
                        <option value="2">2 Week (un-implemented)</option>
                        <option value="3">3 Week (un-implemented)</option>
                        <option value="4">Monthly (un-implemented)</option>
                    </select>
                    <label for="schedule_pattern" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Pattern : </label>
                </div>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium"></span>
                </p>

            </div>
            
            <div class="flex my-5" v-if="form_data.pattern == 0">

                <div class="relative">
                    <select v-model="form_data.shift[0]" class="mt-1 text-gray-700 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="shift_id">
                        <option v-for="d in shift.contents" :value="d.id">{{ d.name }}</option>
                    </select>
                    <label for="shift_id" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Shift : </label>
                </div>
                <p class="text-xs text-red-600 dark:text-red-400">
                    <span class="font-medium"></span>
                </p>

            </div>

            <div class="flex w-full overflow-auto mt-3" v-if="form_data.pattern > 0">

                <div class="w-full min-w-[72lvh] grid grid-cols-8 gap-x-1 gap-y-2">
                
                    <div class="font-medium text-gray-100 rounded-tl-xl bg-red-500">
                        Sun
                    </div>

                    <div class="font-medium text-gray-100 bg-teal-600">
                        Mon
                    </div>

                    <div class="font-medium text-gray-100 bg-teal-600">
                        Tue
                    </div>

                    <div class="font-medium text-gray-100 bg-teal-600">
                        Wed
                    </div>

                    <div class="font-medium text-gray-100 bg-teal-600">
                        Thu
                    </div>

                    <div class="font-medium text-gray-100 bg-teal-600">
                        Fri
                    </div>

                    <div class="font-medium text-gray-100 rounded-tr-xl bg-teal-600">
                        Sat
                    </div>

                    <div class="font-medium text-gray-700 rounded-t-xl bg-purple-300">
                        Copy
                    </div>

                    <div class="font-medium border-2 border-red-500">
                        <NestedSelect :key-id="0" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="1" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="2" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="3" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="4" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="5" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="6" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>
                    
                    <div class="font-medium border-2 border-purple-300">
                        <select>
                            <option>None</option>
                        </select>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="7" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="8" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="9" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="10" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="11" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="12" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="13" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>
                    
                    <div class="font-medium border-2 border-purple-300">
                        <select>
                            <option>None</option>
                            <option>Week 1</option>
                        </select>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="14" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="15" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="16" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="17" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="18" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="19" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="20" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>
                    
                    <div class="font-medium border-2 border-purple-300">
                        <select>
                            <option>None</option>
                            <option>Week 1</option>
                            <option>Week 2</option>
                        </select>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="21" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="22" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="23" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="24" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="25" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="26" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>

                    <div class="font-medium border-2 border-teal-600">
                        <NestedSelect :key-id="27" :def="{key:'Hols', val:0}" :data="shift.contents" v-on:update="optionUpdate"/>
                    </div>
                    
                    <div class="font-medium border-2 border-purple-300">
                        <select>
                            <option>None</option>
                            <option>Week 1</option>
                            <option>Week 2</option>
                            <option>Week 3</option>
                        </select>
                    </div>
                </div>

            </div>

            <!--<div class="flex justify-center">
                Prev / Next
            </div>-->

            <div class="flex mt-3">
                <div class="relative flex justify-start">
                    <button type="submit" class="rounded bg-blue-600 px-10 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700">
                        Save
                    </button>
                    <button type="button" class="ml-4 rounded bg-gray-100 px-10 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-200" v-on:click="">
                        Cancel
                    </button>
                </div>
            </div>
        </form>

    </div>
</template>