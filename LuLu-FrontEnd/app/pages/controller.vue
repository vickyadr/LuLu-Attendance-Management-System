<script setup>
import ListController from '~/components/ListController.vue';
import SimpleModal from '~/components/SimpleModal.vue';
import { useChecker } from '#imports';
import { useAuthStore } from '~/store/auth';
import { useDeviceStore } from '~/store/devices';
import { SMData } from '~/components/SimpleModal.vue';

const
    config = useRuntimeConfig(),
    check = useChecker(),
    auth = useAuthStore(),
    device_store = useDeviceStore(),
    device = reactive({
        id:0,
        name: '',
        sn: '',
        location:'',
        handler: 0,
        timezone: 0,
        }),
    validator = reactive({
        name: '',
        sn: '',
        location:'',
        handler: '',
        timezone: '',
        }),
    sm_data = new SMData(),
    isEdit = ref(false);

useHead({
    title: "LuLu's System - Controller",
});

definePageMeta({
    middleware: ["get-auth"],
    layout: 'default',
});

function actOk(){
    const id = sm_data.getHelper('del');
    if (!check.isNull(id))
        del_controller(id);

    sm_data.clear();

    setTimeout(()=>{
        validator.handler = undefined;
        validator.location = undefined;
        validator.name = undefined;
        validator.sn = undefined;
        validator.timezone = undefined;
    }, 3000);
}

function actCancel(){
    sm_data.clear()
}

function actDelete(id){
    const d = device_store.get(id);
    sm_data.setText("Delete Controller", `${d.sn} will be deleted, are you sure?`);
    sm_data.setHelper('del', id);
    sm_data.showOKCancel();
}

function actEdit(id){
    const d = device_store.get(id);
    device.id = id;
    device.name= d.name;
    device.sn= d.sn;
    device.location= d.location;
    device.handler= d.h_id;
    device.timezone= d.timezone;
    isEdit.value = true;
}

const del_controller = async (id) =>{
   await $fetch(`${config.public.apiBase}/device/delete/${id}`, 
   {
        method: "GET",
        headers: auth.confHeaders(),
   }).then(async (response) => {
        if (response.code == 200)
            device_store.removeList(id);

        sm_data.setText("Delete", response.message);
        sm_data.showOK();
   });
}

const edit_controller = async () =>{

    await $fetch(`${config.public.apiBase}/device/edit`,
        {
            body: {
                id: device.id,
                name: device.name,
                sn: device.sn,
                location: device.location,
                handler: parseInt(device.handler),
                timezone: parseInt(device.timezone),  
            },
            method: "POST",
            headers: auth.confHeaders(),

        }).then(async (response) => {

            if (response.code == 200){
                device_store.updateList(response.data[0]);
                isEdit.value = false;
            }else {
                validator.handler = response.data.handler;
                validator.location = response.data.location;
                validator.sn = response.data.sn;
                validator.name = response.data.name;
                validator.timezone = response.data.timezone;
            }

            sm_data.setText("Edit Controller", response.message);
            sm_data.showOK();
        }
    );
};

const add_controller = async () =>{

    await $fetch(`${config.public.apiBase}/device/add`,
        {
            body: {
                name: device.name,
                sn: device.sn,
                location: device.location,
                handler: parseInt(device.handler),
                timezone: parseInt(device.timezone),  
            },
            method: "POST",
            headers: auth.confHeaders(),

        }).then(async (response) => {

            if (response.code == 200)
                device_store.addList(response.data[0]);
            else {
                validator.handler = response.data.handler;
                validator.location = response.data.location;
                validator.sn = response.data.sn;
                validator.name = response.data.name;
                validator.timezone = response.data.timezone;
            }
            
            sm_data.setText("Add Controller", response.message);
            sm_data.showOK();
        }
    );
};

const device_action = async () =>{
    if (isEdit.value ==true)
        edit_controller();
    else
        add_controller();
}
</script>

<template>
    <div class="h-[90lvh] grid grid-rows-5 grid-cols-5 gap-2">

        <div class="row-span-2 col-span-5">
            <ListController v-on:edit="actEdit" v-on:delete="actDelete"/>
        </div>

        <div class="row-span-3 col-span-5 md:col-span-3 lg:col-span-2 rounded-2xl py-1 border-2 border-purple-100 shadow-sm text-sm text-center">
            <div class="w-full px-4">
                <div class="flex underline text-lg justify-start mt-1 mb-5 font-medium">
                    Add new devices
                </div>
                <form @submit.prevent="device_action">
                <div class="grid grid-cols-2 gap-5">
                    <div class="flex-row">
                        
                        <div class="my-5">
                            <div class="relative">
                                <input v-model="device.name" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" type="text" required="" placeholder=" " id="device_name">
                                <label for="device_name" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Name : </label>
                            </div>

                            <p class="m-2 text-xs text-red-600 dark:text-red-400">
                                <span class="font-medium">{{ validator.name }}</span>
                            </p>
                        </div>

                        <div class="my-5">
                            <div class="relative">
                                <input v-model="device.sn" :disabled="isEdit" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" type="text" required="" placeholder=" " id="serial_number">
                                <label for="serial_number" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Serial Number : </label>
                            </div>

                            <p class="m-2 text-xs text-red-600 dark:text-red-400">
                                <span class="font-medium">{{ validator.sn }}</span>
                            </p>
                        </div>

                        <div class="my-5">
                            <div class="relative">
                                <textarea v-model="device.location" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer resize-none" type="text" placeholder=" " id="location"/>
                                <label for="location" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Location : </label>
                            </div>

                            <p class="m-2 text-xs text-red-600 dark:text-red-400">
                                <span class="font-medium">{{ validator.location }}</span>
                            </p>
                        </div>

                        <div class="my-5">
                            <div class="relative flex justify-start">
                                <button type="submit" class="rounded bg-blue-600 px-10 py-2 text-sm font-medium text-white transition-colors hover:bg-blue-700">
                                    Save
                                </button>
                                <button type="button" class="ml-4 rounded bg-gray-100 px-10 py-2 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-200" v-on:click="isEdit=false" v-show="(isEdit)">
                                    Cancel
                                </button>
                            </div>
                        </div>

                    </div>

                    <div class="flex-row">
                        
                        <div class="my-5">
                            <div class="relative">
                                <input v-model="device.timezone" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" type="text" required="" placeholder=" " id="tz">
                                <label for="tz" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Timezone : </label>
                            </div>

                            <p class="m-2 text-xs text-red-600 dark:text-red-400">
                                <span class="font-medium">{{ validator.timezone }}</span>
                            </p>
                        </div>

                        <div class="my-5">
                            <div class="relative">
                                <select v-model="device.handler" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-sm shadow-sm bg-slate-50 peer" id="handler">
                                    <option value="4">iClock/ADMS</option>
                                    <option value="5">ZKNET (un-implemented)</option>
                                </select>
                                <label for="handler" class="rounded-2xl absolute text-lg scale-90 text-slate-700 transform -translate-y-6 left-1 top-2 z-10 origin-[0] bg-slate-50 px-2">Handler : </label>
                            </div>

                            <p class="m-2 text-xs text-red-600 dark:text-red-400">
                                <span class="font-medium">{{ validator.handler }}</span>
                            </p>
                        </div>
                        
                    </div>

                </div>
                </form>
            </div>
        </div>
        
        <div class="row-span-3 hidden md:block md:col-span-2 lg:col-span-3 flex rounded-2xl py-1 border-2 border-purple-100 shadow-sm text-sm text-center items-center justify-center border-shadow-3px">
            RESERVED AREA FOR COMPLEX CONFIGURATION DEVICES
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" v-on:cancel="actCancel" v-on:ok="actOk" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>