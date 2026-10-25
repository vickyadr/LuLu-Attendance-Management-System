<script setup>
import ListSchedule from '~/components/ListSchedule.vue';
import ListShift from '~/components/ListShift.vue';
import { useChecker } from '#imports';
import { useAuthStore } from '~/store/auth';
import { SMData } from '~/components/SimpleModal.vue';
import { faArrowRight, faCalendarDays, faClock } from '@fortawesome/free-solid-svg-icons';

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

const { t } = useI18n();

useHead({ title: "LuLu - Shifts & Schedules" });
definePageMeta({ middleware: ["get-auth"], layout: 'default' });

function cbDelShift(data) {
    sm_data.setText(t('shift.deleteTitle'), t('shift.deleteBody', { name: data.name }));
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
    <div class="flex flex-col gap-4 lg:flex-1 lg:min-h-0">

        <!-- page head -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-3 shrink-0">
            <div class="min-w-0">
                <h1 class="display text-[20px] text-text">{{ $t('shift.subtitle') }}</h1>
                <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                    <span class="chip chip-quiet">{{ $t('shift.stepCreate') }}</span>
                    <font-awesome :icon="faArrowRight" class="w-2.5 h-2.5 text-text-muted/40 shrink-0" />
                    <span class="chip chip-quiet">{{ $t('shift.stepBuild') }}</span>
                    <font-awesome :icon="faArrowRight" class="w-2.5 h-2.5 text-text-muted/40 shrink-0" />
                    <span class="chip chip-quiet">{{ $t('shift.stepAssign') }}</span>
                </div>
            </div>
        </div>

        <!-- workspace: schedules left, shifts + form right -->
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start lg:items-stretch lg:flex-1 lg:min-h-0">

            <!-- schedule list -->
            <div class="lg:col-span-7 flex flex-col lg:h-full lg:min-h-0">
                <div class="panel overflow-hidden flex flex-col lg:h-full lg:min-h-0">
                    <ListSchedule ref="compScheduleList" />
                </div>
            </div>

            <!-- shift list + form -->
            <div class="lg:col-span-5 lg:h-full lg:min-h-0 lg:overflow-y-auto lg:overscroll-contain space-y-4">

                <div class="panel overflow-hidden flex flex-col lg:min-h-0">
                    <div class="panel-head shrink-0">
                        <span class="inline-flex items-center gap-2 min-w-0">
                            <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                            <span class="truncate">{{ $t('shift.shiftList') }}</span>
                        </span>
                    </div>
                    <div class="lg:max-h-[280px] lg:min-h-0 overflow-auto">
                        <ListShift ref="compShiftList" v-on:delete="cbDelShift" v-on:edit="cbEditShift" />
                    </div>
                </div>

                <div id="shift-form-anchor" class="panel overflow-hidden lg:min-h-0 scroll-mt-4">

                    <div class="flex border-b border-border">
                        <button
                            :class="form_tab === 'schedule' ? 'bg-surface-raised text-primary-deeper border-primary-deep' : 'bg-surface text-text-muted border-transparent hover:bg-surface-raised/50'"
                            class="flex-1 min-h-[44px] px-3 inline-flex items-center justify-center gap-2 border-b-2 mono text-[10.5px] font-bold tracking-[0.1em] uppercase transition"
                            @click="form_tab = 'schedule'">
                            <font-awesome :icon="faCalendarDays" class="w-3 h-3" />
                            {{ $t('shift.createSchedule') }}
                        </button>
                        <button
                            :class="form_tab === 'shift' ? 'bg-surface-raised text-primary-deeper border-primary-deep' : 'bg-surface text-text-muted border-transparent hover:bg-surface-raised/50'"
                            class="flex-1 min-h-[44px] px-3 inline-flex items-center justify-center gap-2 border-b-2 mono text-[10.5px] font-bold tracking-[0.1em] uppercase transition"
                            @click="form_tab = 'shift'">
                            <font-awesome :icon="faClock" class="w-3 h-3" />
                            {{ edit_mode.shift ? $t('shift.editTitleUp') : $t('shift.createTitleUp') }}
                        </button>
                    </div>

                    <div v-if="edit_mode.shift"
                        class="px-4 py-2 bg-warn-bg border-b border-amber-200 flex items-center justify-between gap-2">
                        <span class="mono text-[10px] font-bold tracking-[0.14em] text-amber-700">{{
                            $t('shift.editMode') }}</span>
                        <button @click="shiftEditDone(); compShiftForm?.fillForm && (compShiftForm.fillForm({ id: 0 }))"
                            class="btn btn-secondary">{{ $t('common.cancelEdit') }}</button>
                    </div>

                    <div v-show="form_tab === 'schedule'">
                        <FormSchedule ref="compScheduleForm" @done="onScheduleDone" />
                    </div>
                    <div v-show="form_tab === 'shift'">
                        <FormShift ref="compShiftForm" @done-edit="shiftEditDone" />
                    </div>
                </div>

            </div>
        </div>
    </div>

    <SimpleModal :options="sm_data.get()" @ok="actOk" @cancel="actCancel" v-show="sm_data.isShow()" />
</template>
