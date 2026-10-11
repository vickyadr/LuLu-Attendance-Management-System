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
    compShiftList = ref(),
    compShiftForm = ref(),
    compScheduleList = ref(),
    compScheduleForm = ref(),
    edit_mode = ref({ shift: false }),
    sm_data = new SMData();

useHead({ title: "LuLu - Shifts & Schedules" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

function cbDelShift(data) {
    sm_data.setText("Delete Shift", `Shift "${data.name}" will be deleted. Every schedule using it will fall back to Day off.`);
    sm_data.setHelper("del-shift", data);
    sm_data.showOKCancel();
}
function cbEditShift(data) {
    compShiftList.value?.reload?.()
    compShiftForm.value?.fillForm(data)
    form_tab.value = 'shift'
    edit_mode.value.shift = true
    // scroll to form on mobile
    nextTick(() => document.getElementById('shift-form-anchor')?.scrollIntoView({ behavior: 'smooth', block: 'start' }))
}
function actOk() {
    const del_shift = sm_data.getHelper("del-shift")
    if (!check.isNull(del_shift)) compShiftForm.value?.del_shift(del_shift.id)
    sm_data.clear()
}
function actCancel() { sm_data.clear() }
function shiftEditDone() { edit_mode.value.shift = false; form_tab.value = 'schedule' }
function onScheduleDone() { /* after create schedule, stay on schedule tab */ }
</script>

<template>
    <!-- The desktop layout keeps <main> scrolling (see layouts/default.vue), but this
         page pins its own height: the schedule list gets an inner scrollbar and the
         right column (shift list + form) stays put while you scroll through
         schedules. Without a bounded height the grid just grows and the whole page
         scrolls, which drags the right-hand cards out of view. -->
    <div class="flex flex-col gap-4 lg:flex-1 lg:min-h-0">
        <!-- Page header -->
        <div
            class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col sm:flex-row sm:items-center gap-3 tech-grid-soft shrink-0">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3">
                <div
                    class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8">
                        <rect x="3" y="4" width="18" height="18" rx="2" />
                        <path d="M16 2v4M8 2v4M3 10h18" />
                    </svg>
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">{{ $t('shift.subtitle') }}
                    </h1>
                    <div class="mt-2 flex flex-wrap gap-1.5 mono text-[10px] tracking-widest">
                        <span class="chip chip-emerald">{{ $t('shift.stepCreate') }}</span>
                        <span class="text-emerald-700/40">→</span>
                        <span class="chip chip-emerald">{{ $t('shift.stepBuild') }}</span>
                        <span class="text-emerald-700/40">→</span>
                        <span class="chip bg-white border-emerald-200 text-emerald-700 border">{{ $t('shift.stepAssign')
                        }}</span>
                    </div>
                </div>
            </div>
            <div class="ml-auto flex items-center gap-2 mono text-[10px]">
            </div>
        </div>

        <!-- Main grid: left schedules (self-scrolling), right shifts+form (pinned) -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start lg:items-stretch lg:flex-1 lg:min-h-0">
            <!-- LEFT: Schedule list (takes 7 cols) - the inner list (ListSchedule)
                 already has flex-1 + overflow-auto, so bounding this column pins the
                 scroll here instead of pushing the whole page. -->
            <div class="lg:col-span-7 xl:col-span-7 lg:h-full lg:min-h-0 flex flex-col">
                <div
                    class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden flex flex-col lg:h-full lg:min-h-0">
                    <ListSchedule ref="compScheduleList" />
                </div>
            </div>

            <!-- RIGHT: Shift list + Form tabs (5 cols). This column scrolls on its own so
                 the left schedule list can scroll without dragging these cards
                 along, and the page height stays fixed. -->
            <div
                class="lg:col-span-5 xl:col-span-5 lg:h-full lg:min-h-0 lg:overflow-y-auto lg:overscroll-contain space-y-4">
                <!-- Shift list card. The inner list flexes with the column instead of
                     a hard 320px cap: with a short viewport the form below would sit
                     far under the fold, and the user has to scroll to reach it. -->
                <div
                    class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden flex flex-col lg:min-h-0">
                    <div
                        class="px-4 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/60 to-white flex items-center justify-between shrink-0">
                        <h2 class="mono text-xs font-bold tracking-[0.14em] text-emerald-700 flex items-center gap-2">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span> {{ $t('shift.shiftList') }}
                        </h2>
                    </div>
                    <div class="lg:max-h-[220px] lg:min-h-0 overflow-auto">
                        <ListShift ref="compShiftList" v-on:delete="cbDelShift" v-on:edit="cbEditShift" />
                    </div>
                </div>

                <!-- Form tabs. When the viewport is short the column scrolls past them; the
                     shift list above is capped for exactly that reason. -->
                <div id="shift-form-anchor"
                    class="glass rounded-2xl border border-emerald-100 shadow-sm overflow-hidden lg:min-h-0">
                    <div class="flex border-b border-emerald-100 bg-white/60">
                        <button
                            :class="form_tab === 'schedule' ? 'bg-emerald-600 text-white shadow-sm' : 'bg-white text-emerald-700 hover:bg-emerald-50'"
                            class="flex-1 py-3 px-4 text-xs font-bold mono tracking-widest flex items-center justify-center gap-2 transition"
                            @click="form_tab = 'schedule'">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.8">
                                <rect x="3" y="4" width="18" height="18" rx="2" />
                                <path d="M16 2v4M8 2v4M3 10h18" />
                            </svg>
                            {{ $t('shift.createSchedule') }}
                        </button>
                        <button
                            :class="form_tab === 'shift' ? 'bg-emerald-600 text-white shadow-sm' : 'bg-white text-emerald-700 hover:bg-emerald-50'"
                            class="flex-1 py-3 px-4 text-xs font-bold mono tracking-widest flex items-center justify-center gap-2 transition border-l border-emerald-100"
                            @click="form_tab = 'shift'">
                            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                stroke-width="1.8">
                                <circle cx="12" cy="12" r="9" />
                                <path d="M12 8v4l3 2" />
                            </svg>
                            {{ edit_mode.shift ? $t('shift.editTitleUp') : $t('shift.createTitleUp') }}
                        </button>
                    </div>
                    <div v-if="edit_mode.shift"
                        class="px-4 py-2 bg-amber-50 border-b border-amber-200 flex items-center justify-between">
                        <span class="mono text-[10px] font-bold tracking-widest text-amber-700">{{ $t('shift.editMode')
                        }}</span>
                        <button @click="shiftEditDone(); compShiftForm?.fillForm && (compShiftForm.fillForm({ id: 0 }))"
                            class="mono text-[10px] px-2.5 py-1 rounded-full bg-white border border-amber-200 text-amber-700 hover:bg-amber-50">{{
                                $t('common.cancelEdit') }}</button>
                    </div>

                    <div v-show="form_tab === 'schedule'" class="bg-white">
                        <FormSchedule ref="compScheduleForm" @done="onScheduleDone" />
                    </div>
                    <div v-show="form_tab === 'shift'" class="bg-white">
                        <FormShift ref="compShiftForm" @done-edit="shiftEditDone" />
                    </div>
                </div>

            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" @ok="actOk" @cancel="actCancel" v-show="sm_data.isShow()">
        {{ sm_data.message() }}
    </SimpleModal>
</template>
