<script setup>
import ListSchedule from '~/components/ListSchedule.vue';
import ListShift from '~/components/ListShift.vue';
import { useChecker, useFormater } from '#imports';
import { useAuthStore } from '~/store/auth';
import { SMData } from '~/components/SimpleModal.vue';

const
    form_shift = ref(false),
    config = useRuntimeConfig(),
    check = useChecker(),
    format = useFormater(),
    auth = useAuthStore(),
    compShift = ref(),
    edit_mode = ref({
        shift: false,
        schedule:false
    }),
    compSchedule = ref(),
    sm_data = new SMData();

useHead({
    title: "LuLu's System - Schedule",
});

definePageMeta({
    middleware: ["get-auth"],
    layout: 'default',
});

function cbDelShift(data){
    sm_data.setText("Delete Shift", `Shift "${data.name}" will be deleted, are you sure?`);
    sm_data.setHelper("del-shift", data);
    sm_data.showOKCancel();
}

function cbEditShift(data){
    compShift.value?.fillForm(data)
    form_shift.value = true
    edit_mode.value.shift = true
}

function actOk(){
    const del_shift = sm_data.getHelper("del-shift")
    if (!check.isNull(del_shift))
        compShift.value?.del_shift(del_shift.id)
    sm_data.clear()
}

function actCancel(){
    sm_data.clear()
}

function shiftEditDone(){
    edit_mode.value.shift = false
}

function showModal(data){
    sm_data.setText(data.title, data.message)
    sm_data.showOK()
}
</script>

<template>
        <div class="h-[90lvh] grid grid-rows-5 grid-cols-2 gap-2">
        
            <div class="row-span-5 rounded-2xl py-1 border-2 border-purple-100 shadow-sm text-sm">
                <ListSchedule/>
            </div>

            <div class="row-span-2 rounded-2xl border-2 border-purple-100 shadow-sm text-sm text-center">
                <div class="flex underline text-lg justify-start my-2 ml-4 font-medium text-gray-600">
                    List shift
                </div>
                <div class="overflow-auto">
                    <ListShift v-on:delete="cbDelShift" v-on:edit="cbEditShift"/>
                </div>
            </div>

            <div class="row-span-3 rounded-2xl border-2 border-purple-100 shadow-sm text-sm text-center">
                
                <div class="flex border-b-2 shadow-sm">
                    <button v-bind:class="(!form_shift ? 'bg-teal-600 text-gray-100' : 'bg-teal-200 text-gray-500')"  class="py-2 px-4 rounded-tl-xl font-bold text-nowrap" v-on:click="()=>{form_shift = false}">
                        Schedule
                    </button>
                    <button v-bind:class="(form_shift ? 'bg-teal-600 text-gray-100' : 'bg-teal-200 text-gray-500')" class="py-2 px-10 rounded-tr-xl font-bold border-l-2 border-slate-300" v-on:click="()=>form_shift = true">
                        Shift                  
                    </button>
                    <div class="flex items-center w-full justify-end text-right mr-2">
                        <div class="bg-red-300 rounded-xl px-2 uppercase font-medium" v-if="edit_mode.shift">SHIFT EDIT</div>
                        <div class="bg-red-300 rounded-xl px-2 uppercase font-medium" v-if="edit_mode.schedule">SCHEDULE EDIT</div>
                    </div>
                </div>

                <div class="px-0" v-show="!form_shift">
                    <FormSchedule/>
                </div>

                <div class="px-0" v-show="form_shift">
                    <FormShift ref="compShift" v-on:done-edit="shiftEditDone" v-on:notif="showModal"/>
                </div>
            </div>
        </div>
        
    <SimpleModal :options="sm_data.get()" v-on:ok="actOk" v-on:cancel="actCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>