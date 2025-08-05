<script setup>
import LiveTable from '~/components/LiveTable.vue';
import SimpleModal from '~/components/SimpleModal.vue';

useHead({
    title: "LuLu's System - Dashboard",
});

definePageMeta({
    middleware: ["get-auth"],
    layout: 'default',
});
</script>

<template>

    <div class="h-[90lvh] max-w grid grid-cols-4 grid-rows-5 gap-2">

        <div class="flex rounded-2xl py-1 row-span-2 border-2 border-purple-100 shadow-sm text-sm text-center items-center justify-center border-shadow-3px">
            NEW EMPLOYEE
        </div>

        <div class="flex rounded-2xl py-1 col-span-3 row-span-2 border-2 border-purple-100 shadow-sm text-sm text-center items-center justify-center">
            STATISTICS
        </div>
    
        <div class="rounded-2xl py-1 col-span-4 row-span-3 border-2 border-purple-100 shadow-sm">

            <h1 class="underline text-lg mt-1 mb-2 px-4 font-medium">Live Transaction</h1>
            <LiveTable/>
        </div>

    </div>
</template>