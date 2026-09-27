<script setup>
const ba = ref(false)
async function sleep() {
  await new Promise(resolve => setTimeout(resolve, 3000));
  ba.value = true;
  return 0;
}
onMounted(() => { sleep(); })
useHead(() => ({
  status: 404,
  title: "Page not found — LuLu System",
}));
</script>

<template>
    <div class="min-h-[60vh] grid place-content-center text-center p-8 tech-grid-soft relative">
        <div class="hud-frame glass rounded-[2rem] p-10 border border-emerald-100 max-w-md mx-auto relative overflow-hidden">
            <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span><div class="mono text-[10px] tracking-widest text-emerald-700/40 mb-2">ERR 404 • ROUTE NOT FOUND</div><div class="mx-auto w-14 h-14 rounded-xl bg-emerald-50 border border-emerald-200 flex items-center justify-center text-emerald-600 mb-4"><svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.8"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z"/></svg></div>
            <h1 class="text-4xl font-extrabold text-green-900">404</h1>
            <p class="text-emerald-700/70 mt-2 font-medium">Page not found</p>
            <p class="text-sm text-emerald-700/50 mt-1">You are lost — head back to the dashboard?</p>
            <button v-on:click="navigateTo('/', {replace:true})" class="mt-6 btn-emerald px-6 py-2.5 rounded-xl text-sm font-bold">Back to Dashboard</button>
            <p v-if="!ba" class="text-xs text-emerald-700/40 mt-4">Redirecting in 3 seconds…</p>
        </div>
    </div>
</template>
