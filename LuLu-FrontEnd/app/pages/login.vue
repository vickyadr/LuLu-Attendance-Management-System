<script setup>
import { useAuthStore } from '~/store/auth';
import { LiveTable } from '#components';
import { SMData } from '~/components/SimpleModal.vue';

const
    validation = reactive({user_id:'', password: '',}),
    user = reactive({
        user_id: '', password: ''
    }),
    auth = useAuthStore(),
    sm_data = new SMData();

useHead({
    title: "LuLu's System - Login Page",
});

definePageMeta({
    middleware: ["get-auth"],
    layout: 'none',
});

const login = async () => {

    await auth.attemptLogin({
        nickname: user.user_id,
        password: user.password,
        cookies: true,
    }).then((response) => {
        if (response.code == 200) {
            sm_data.setText("Login", "Login succeess, please wait we're tryng to redirecting the page")
            sm_data.showOK()
            setTimeout(() => {
                navigateTo("/", {replace: true });
                sm_data.clear();
                }, 3000)
        } else {            
            validation.user_id = response.data.nickname;
            validation.password = response.data.password;
            sm_data.setText("Login", response.message)
            sm_data.showOK()
        }
    })
}

function actOk() {
    sm_data.clear()
    setTimeout(()=>{
        validation.user_id = undefined;
        validation.password = undefined;
    }, 3000);
}

</script>

<template>
    <div class="flex items-center justify-center bg-gradient-to-br from-slate-100 to-slate-300 font-mono min-h-screen">
        <div class="bg-gradient-to-b from-slate-100 to-slate-300 rounded-2xl p-5 m-5 border-2 border-slate-100 shadow-2xl">
            
            <div class="flex justify-center mt-2 p-2">
                <h1 class="text-3xl uppercase font-bold">LuLu Attendance System</h1>
            </div>
            
            <div class="flex justify-center mt-2 mb-10 p-2">
                <h3 class="text-lg">Manage with no limitation</h3>
            </div>

            <form @submit.prevent="login">
            
                <div class="my-10">
                    <div class="relative">
                        <input v-model="user.user_id" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-2xl shadow-sm bg-slate-50 peer" type="text" required="" placeholder=" " id="user_name">
                        <label for="user_name" class="rounded-2xl absolute text-2xl text-slate-400 duration-300 transform -translate-y-7 scale-80 top-2 z-10 origin-[0] bg-slate-50 px-2 peer-focus:px-2 peer-placeholder-shown:scale-100 peer-placeholder-shown:-translate-y-1/2 peer-placeholder-shown:top-1/2 peer-focus:top-2 peer-focus:scale-80 peer-focus:-translate-y-7 left-1">User ID</label>
                    </div>

                    <p v-if="!!!validation.user_id != undefined" class="m-2 text-sm text-red-600 dark:text-red-400">
                        <span class="font-medium">{{ validation.user_id }}</span>
                    </p>
                </div>
                
                <div class="my-10">
                    <div class="relative">
                        <input v-model="user.password" class="text-gray-700 mt-1 block w-full px-3 py-2 border-2 duration-500 focus:border-purple-500 border-purple-200 hover:border-purple-300 rounded-md text-2xl shadow-sm bg-slate-50 peer" type="password" required="" autocomplete="" placeholder=" " id="user_password">
                        <label for="user_password" class="rounded-2xl absolute text-2xl text-slate-400 duration-300 transform -translate-y-7 scale-80 top-2 z-10 origin-[0] bg-slate-50 px-2 peer-focus:px-2 peer-placeholder-shown:scale-100 peer-placeholder-shown:-translate-y-1/2 peer-placeholder-shown:top-1/2 peer-focus:top-2 peer-focus:scale-80 peer-focus:-translate-y-7 left-1">Password</label>
                    </div>

                    <p v-if="!!!validation.password != undefined" class="m-2 text-sm text-red-600 dark:text-red-400">
                        <span class="font-medium">{{ validation.password }}</span>
                    </p>
                </div>
                
                <div class="my-10">
                    <div class="relative">
                        <button class="w-full rounded-2xl border-2 border-slate-100 text-2xl p-2 hover:border-purple-300 duration-500 hover:bg-gradient-to-r hover:from-slate-200 hover:via-amber-200 hover:to-slate-200">
                            <p>LOGIN</p>
                        </button>
                    </div>
                </div>
            </form>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" v-on:ok="actOk" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>