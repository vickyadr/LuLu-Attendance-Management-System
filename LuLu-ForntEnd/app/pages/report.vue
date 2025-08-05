<script setup lang="ts">

definePageMeta({
    middleware: ["get-auth"],
    layout: 'default',
});

</script>

<template>
    <div class="h-[90lvh] border-2 rounded-xl border-purple-100 shadow-sm">

        <div class="">
            <div class="flex justify-between items-center pr-5 pl-2">
                <div class="flex justify-between text-nowrap text-gray-700">
                    <span>Period :</span>
                    <div class="flex justify-center px-2 mx-2 border-2 border-purple-100 hover:border-purple-200 rounded-lg">
                        <input class="outline-transparent scale-90 w-7" placeholder="DD" maxlength="2"/>
                        <span>/</span>
                        <input class="outline-transparent scale-90 w-7" placeholder="MM" maxlength="2"/>
                        <span>/</span>
                        <input class="outline-transparent scale-90 w-10" placeholder="YYYY" maxlength="4"/>
                    </div>
                    <span>to</span> 
                    <div class="flex justify-center px-2 mx-2 border-2 border-purple-100 hover:border-purple-200 rounded-lg">
                        <input class="outline-transparent scale-90 w-7" placeholder="DD" maxlength="2"/>
                        <span>/</span>
                        <input class="outline-transparent scale-90 w-7" placeholder="MM" maxlength="2"/>
                        <span>/</span>
                        <input class="outline-transparent scale-90 w-10" placeholder="YYYY" maxlength="4"/>
                    </div>         
                </div>

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
        </div>

        <ListReport/>
    </div>
</template>