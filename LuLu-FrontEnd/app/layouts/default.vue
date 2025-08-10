<script setup>
import { useAuthStore } from '~/store/auth';
import { faHouse, faCalendarCheck } from '@fortawesome/free-regular-svg-icons'
import { faMicrochip, faBusinessTime } from '@fortawesome/free-solid-svg-icons'
import logo from '~/assets/img/android-chrome-512x512.png'
import { SMData } from '~/components/SimpleModal.vue';

const open_menu_m = ref(false);

const auth = useAuthStore(),
    sm_data = new SMData();

function actCancel() {
    sm_data.clear()
}

function actOk() {
    sm_data.clear()
    auth.logout()
}

function modalLogout(){
    sm_data.setText("Logout", "Are you sure want to logout ?")
    sm_data.showOKCancel()
}
</script>

<template>
    <div class="max-w flex flex-row">
        <div class="h-screen w-16 flex-col justify-between border-e border-gray-100 bg-white shadow-md hidden sm:flex">
            <div>
            <div class="inline-flex size-16 items-center justify-center">
                <img :src="logo" class="grid size-10 place-content-center rounded-lg bg-gray-100 text-xs text-gray-600"/>
            </div>
        
            <div class="border-t border-gray-100">
                <div class="px-2">
                
                    <div class="py-4">
                    <a
                    href="#"
                    class="t group relative flex justify-center rounded-sm bg-blue-50 px-2 py-1.5 text-blue-700"
                    >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="size-5 opacity-75"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        />
                        <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                    </svg>
        
                    <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                    >
                        General
                    </span>
                    </a>
                </div>
        
                <ul class="space-y-1 border-t border-gray-100 pt-4">
                    <li>
                    <a
                        v-on:click="navigateTo('/', {replace: true})"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <font-awesome :icon="faHouse"/>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Dashboard
                        </span>
                    </a>
                    </li>

                    <li>
                    <a
                        v-on:click="navigateTo('/controller', {replace: true})"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <font-awesome :icon="faMicrochip"/>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Controller
                        </span>
                    </a>
                    </li>
            
                    <li>
                    <a
                        v-on:click="navigateTo('/shift', {replace: true})"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <font-awesome :icon="faBusinessTime"/>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Schedule
                        </span>
                    </a>
                    </li>

                    <li>
                    <a
                        v-on:click="navigateTo('/report', {replace: true})"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <font-awesome :icon="faCalendarCheck"/>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Report
                        </span>
                    </a>
                    </li>
                    
                    <li>
                    <a
                        v-on:click="navigateTo('/employee', {replace: true})"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="size-5 opacity-75"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                        >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"
                        />
                        </svg>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Employee
                        </span>
                    </a>
                    </li>
        
                    <li>
                    <a
                        href="#"
                        class="group relative flex justify-center rounded-sm px-2 py-1.5 text-gray-500 hover:bg-gray-50 hover:text-gray-700"
                    >
                        <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="size-5 opacity-75"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                        >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"
                        />
                        </svg>
        
                        <span
                        class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                        >
                        Payroll
                        </span>
                    </a>
                    </li>

                </ul>
                </div>
            </div>
            </div>

            <div class="sticky inset-x-0 bottom-0 border-t border-gray-100 bg-white p-2">
            <a
                href="#"
                v-on:click="modalLogout"
                class="group relative flex w-full justify-center rounded-lg px-2 py-1.5 text-sm text-gray-500 hover:bg-gray-50 hover:text-gray-700"
            >
                <svg
                xmlns="http://www.w3.org/2000/svg"
                class="size-5 opacity-75"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="2"
                >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                />
                </svg>
        
                <span
                class="z-10 invisible absolute start-full top-1/2 ms-4 -translate-y-1/2 rounded-sm bg-gray-900 px-2 py-1.5 text-xs font-medium text-white group-hover:visible"
                >
                Logout
                </span>
            </a>
            </div>
        </div>

        <div class="flex-col w-full">
            <header class="bg-white shadow-md">
                <div class="mx-auto max-w px-4 sm:px-6 lg:px-8">
                <div class="flex h-16 items-center justify-end">
            
                    <div class="md:flex md:items-center md:gap-12">
                    <nav aria-label="Global" class="hidden sm:block">
                        <ul class="flex items-center gap-6 text-sm">
                        <li>
                            <a class="text-gray-500 transition hover:text-gray-500/75" href="#"> Storage </a>
                        </li>
            
                        <li>
                            <a class="text-gray-500 transition hover:text-gray-500/75" href="#"> Statistic </a>
                        </li>
            
                        <li>
                            <a class="text-gray-500 transition hover:text-gray-500/75" href="#"> About </a>
                        </li>
                        </ul>
                    </nav>
            
                    <div class="flex items-center gap-4">

                        <div class="block sm:hidden">
                            <button
                                v-on:click="open_menu_m = !open_menu_m"
                                class="rounded-sm bg-gray-100 p-2 text-gray-600 transition hover:text-gray-600/75"
                            >
                                <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="size-5"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                stroke-width="2"
                                >
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
                                </svg>
                            </button>
                        
                            <div v-show="open_menu_m" class="fixed inset-0 mt-16 z-50 bg-slate-100 w-full max-h-[92lvh]">
                                <div>
                                    <a v-on:click="()=> {navigateTo('/', {replace: true}); open_menu_m = false;}"
                                        class="flex py-1.5 text-gray-700 hover:bg-gray-50 hover:text-gray-500 items-center">
                                        <font-awesome :icon="faHouse" class="px-2"/>
                                        <span>
                                            Dashboard
                                        </span>
                                    </a>

                                    <a v-on:click="()=> {navigateTo('/controller', {replace: true}); open_menu_m = false;}"
                                        class="flex py-1.5 text-gray-700 hover:bg-gray-50 hover:text-gray-500 items-center">
                                        <font-awesome :icon="faMicrochip" class="px-2"/>
                                        <span>
                                            Controller
                                        </span>
                                    </a>

                                    <a v-on:click="()=> {navigateTo('/shift', {replace: true}); open_menu_m = false;}"
                                        class="flex py-1.5 text-gray-700 hover:bg-gray-50 hover:text-gray-500 items-center">
                                        <font-awesome :icon="faBusinessTime" class="px-2"/>
                                        <span>
                                            Schedule
                                        </span>
                                    </a>

                                    <a v-on:click="()=> {navigateTo('/report', {replace: true}); open_menu_m = false;}"
                                        class="flex py-1.5 text-gray-700 hover:bg-gray-50 hover:text-gray-500 items-center">
                                        <font-awesome :icon="faCalendarCheck" class="px-2"/>
                                        <span>
                                            Report
                                        </span>
                                    </a>
                                    <hr class="border-2 mx-4"/>
                                    <a v-on:click="modalLogout"
                                        class="flex py-3 px-2 text-gray-700 hover:bg-gray-50 hover:text-gray-500 items-center">
                                        <svg
                                        xmlns="http://www.w3.org/2000/svg"
                                        class="size-5 opacity-75 text-red-500"
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                                        />
                                        </svg>
                                        <span class="text-teal-700 px-2">
                                            Logout
                                        </span>
                                    </a>
                                </div>
                            </div>
                        </div>
                    </div>
                    </div>
                </div>
                </div>
            </header>
            <div class="pt-2 px-2">
                <slot/>
            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" v-on:cancel="actCancel" v-on:ok="actOk" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>