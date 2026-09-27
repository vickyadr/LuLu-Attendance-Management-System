<script setup>
import ListSchedule from '~/components/ListSchedule.vue';
import ListShift from '~/components/ListShift.vue';
import { useChecker } from '#imports';
import { useAuthStore } from '~/store/auth';
import { SMData } from '~/components/SimpleModal.vue';

const
    form_tab = ref('schedule'), // 'schedule' | 'shift'
    config = useRuntimeConfig(),
    check = useChecker(),
    auth = useAuthStore(),
    compShift = ref(),
    compSchedule = ref(),
    edit_mode = ref({ shift: false }),
    sm_data = new SMData();

useHead({ title: "LuLu — Shifts & Schedules" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

function cbDelShift(data){
    sm_data.setText("Delete Shift", `Shift "${data.name}" will be deleted. Every schedule using it will fall back to Day off.`);
    sm_data.setHelper("del-shift", data);
    sm_data.showOKCancel();
}
function cbEditShift(data){
    compShift.value?.fillForm(data)
    form_tab.value = 'shift'
    edit_mode.value.shift = true
    // scroll to form on mobile
    nextTick(()=> document.getElementById('shift-form-anchor')?.scrollIntoView({ behavior:'smooth', block:'start' }))
}
function actOk(){
    const del_shift = sm_data.getHelper("del-shift")
    if (!check.isNull(del_shift)) compShift.value?.del_shift(del_shift.id)
    sm_data.clear()
}
function actCancel(){ sm_data.clear() }
function shiftEditDone(){ edit_mode.value.shift = false; form_tab.value = 'schedule' }
function showModal(data){ sm_data.setText(data.title, data.message); sm_data.showOK() }
function onScheduleDone(){ /* after create schedule, stay on schedule tab */ }
</script>

<template>
    <div class="space-y-4">
        <!-- Page header -->
        <div class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col sm:flex-row sm:items-center gap-3 tech-grid-soft">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3">
                <div class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">Shifts & Work Schedules</h1>
                    <p class="text-xs text-emerald-700/60 mt-0.5">Create a <b>Shift</b> (work hours) → build the <b>schedule</b> daily / weekly / monthly → then use it in the employee records.</p>
                    <div class="mt-2 flex flex-wrap gap-1.5 mono text-[10px] tracking-widest">
                        <span class="chip chip-emerald">1. CREATE SHIFT</span>
                        <span class="text-emerald-700/40">→</span>
                        <span class="chip chip-emerald">2. BUILD THE SCHEDULE</span>
                        <span class="text-emerald-700/40">→</span>
                        <span class="chip bg-white border-emerald-200 text-emerald-700 border">3. ASSIGN TO EMPLOYEES</span>
                    </div>
                </div>
            </div>
            <div class="ml-auto flex items-center gap-2 mono text-[10px]">
                <span class="hidden md:inline-flex items-center gap-1.5 chip bg-white border-emerald-200 text-emerald-700 border"><span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span> DRAG & DROP ACTIVE</span>
                <span class="hidden sm:inline text-emerald-700/40">Drag a shift onto the schedule</span>
            </div>
        </div>

        <!-- Main grid: left schedules, right shifts+form -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
            <!-- LEFT: Schedule list (takes 7 cols) -->
            <div class="lg:col-span-7 xl:col-span-7">
                <div class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden">
                    <ListSchedule ref="compSchedule" @notif="showModal" />
                </div>
                <p class="mt-2 mono text-[10px] tracking-wide text-emerald-700/40 px-1">Tip: click a schedule name to expand/collapse • drag a shift from the right onto a day to change that day</p>
            </div>

            <!-- RIGHT: Shift list + Form tabs (5 cols) -->
            <div class="lg:col-span-5 xl:col-span-5 space-y-4">
                <!-- Shift list card -->
                <div class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden">
                    <div class="px-4 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-white flex items-center justify-between">
                        <h2 class="mono text-xs font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> SHIFT LIST
                        </h2>
                        <span class="mono text-[10px] tracking-widest text-emerald-700/40 hidden sm:inline">Drag to assign</span>
                    </div>
                    <div class="max-h-[320px] overflow-auto">
                        <ListShift v-on:delete="cbDelShift" v-on:edit="cbEditShift" />
                    </div>
                </div>

                <!-- Form tabs -->
                <div id="shift-form-anchor" class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden">
                    <div class="flex border-b border-emerald-100 bg-white/60">
                        <button :class="form_tab==='schedule' ? 'bg-emerald-600 text-white shadow-sm' : 'bg-white text-emerald-700 hover:bg-emerald-50'" class="flex-1 py-3 px-4 text-xs font-bold mono tracking-widest flex items-center justify-center gap-2 transition" @click="form_tab='schedule'">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>
                            CREATE SCHEDULE
                        </button>
                        <button :class="form_tab==='shift' ? 'bg-emerald-600 text-white shadow-sm' : 'bg-white text-emerald-700 hover:bg-emerald-50'" class="flex-1 py-3 px-4 text-xs font-bold mono tracking-widest flex items-center justify-center gap-2 transition border-l border-emerald-100" @click="form_tab='shift'">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="12" cy="12" r="9"/><path d="M12 8v4l3 2"/></svg>
                            {{ edit_mode.shift ? 'EDIT SHIFT' : 'CREATE SHIFT' }}
                        </button>
                    </div>
                    <div v-if="edit_mode.shift" class="px-4 py-2 bg-amber-50 border-b border-amber-200 flex items-center justify-between">
                        <span class="mono text-[10px] font-bold tracking-widest text-amber-700">EDIT MODE — make your changes, then Save Changes</span>
                        <button @click="shiftEditDone(); compShift?.fillForm && (compShift.fillForm({id:0}))" class="mono text-[10px] px-2.5 py-1 rounded-full bg-white border border-amber-200 text-amber-700 hover:bg-amber-50">Cancel edit</button>
                    </div>

                    <div v-show="form_tab==='schedule'" class="bg-white">
                        <FormSchedule ref="compScheduleForm" @notif="showModal" @done="onScheduleDone" />
                    </div>
                    <div v-show="form_tab==='shift'" class="bg-white">
                        <FormShift ref="compShift" @done-edit="shiftEditDone" @notif="showModal" />
                    </div>
                </div>

                <div class="rounded-xl bg-emerald-50/60 border border-emerald-100 px-3.5 py-3 mono text-[10px] leading-relaxed text-emerald-800">
                    <p class="font-bold tracking-widest">QUICK START</p>
                    <ol class="mt-1.5 list-decimal list-inside space-y-1 text-emerald-700/80">
                        <li>Create <b>Shift</b> (e.g. Morning 08–17) in the “Create Shift” tab.</li>
                        <li>Switch to “Create Schedule” → pick a Daily/Weekly/Monthly pattern → fill in a shift per day.</li>
                        <li>Open the <b>Employees</b> menu → pick a schedule for each employee.</li>
                    </ol>
                </div>
            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" @ok="actOk" @cancel="actCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>
