<script setup>
import { useSettingsStore } from '~/store/settings'
import { useAuthStore } from '~/store/auth'
import { faSliders } from '@fortawesome/free-solid-svg-icons'
import { useNotifyStore } from '~/store/notify'

const settings = useSettingsStore()
const auth = useAuthStore()
const notify = useNotifyStore()
const { t } = useI18n()

useHead({ title: 'LuLu - Settings' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

const lateTolMin = ref(1)
const overtimeTolMin = ref(1)
const saving = ref(false)

const canSave = computed(() => !saving.value && lateTolMin.value >= 0 && lateTolMin.value <= 60 && overtimeTolMin.value >= 0 && overtimeTolMin.value <= 60)

// Values come from the backend and are converted to minutes for the input; the DB stores seconds.
onMounted(async () => {
    try {
        const cur = await settings.fetchSettings()
        lateTolMin.value = Math.round((cur.late_tolerance_sec || 0) / 60)
        overtimeTolMin.value = Math.round((cur.overtime_tolerance_sec || 0) / 60)
    } catch (e) {
        notify.error(t('common.saveFailed'), e?.message || t('common.networkError'))
    }
})

async function save() {
    if (!canSave.value) return
    saving.value = true
    try {
        const resp = await settings.saveSettings({
            late_tolerance_sec: Math.round(lateTolMin.value * 60),
            overtime_tolerance_sec: Math.round(overtimeTolMin.value * 60),
        })
        if (resp.code === 200) {
            notify.success(t('settings.savedTitle'), t('settings.savedMsg'))
        } else {
            notify.error(t('common.saveFailed'), resp.message || '')
        }
    } catch (e) {
        notify.error(t('common.networkError'), t('settings.backendRunning'))
    }
    saving.value = false
}
</script>

<template>
    <div class="flex flex-col gap-4">

        <!-- ===== page head ===== -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-3">
            <div class="min-w-0">
                <h1 class="display text-[20px] text-text">{{ $t('settings.attendance') }}</h1>
                <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
                    <span class="chip chip-quiet">{{ $t('settings.thresholds') }}</span>
                </div>
            </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">

            <!-- ===== tolerances ===== -->
            <div class="lg:col-span-7 panel overflow-hidden">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t('settings.attendance') }}</span>
                    </span>
                    <font-awesome :icon="faSliders" class="w-3.5 h-3.5 shrink-0" />
                </div>

                <div class="panel-body space-y-4">
                    <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                        <div>
                            <label for="late-tol" class="label">{{ $t('settings.lateTolerance') }}</label>
                            <div class="flex items-center gap-2">
                                <input id="late-tol" v-model.number="lateTolMin" type="number" min="0" max="60" step="1"
                                    class="field !w-24 mono">
                                <span class="text-[12px] text-text-muted">{{ $t('settings.minutes') }}</span>
                            </div>
                        </div>

                        <div>
                            <label for="overtime-tol" class="label">{{ $t('settings.overtimeTolerance') }}</label>
                            <div class="flex items-center gap-2">
                                <input id="overtime-tol" v-model.number="overtimeTolMin" type="number" min="0" max="60"
                                    step="1" class="field !w-24 mono">
                                <span class="text-[12px] text-text-muted">{{ $t('settings.minutes') }}</span>
                            </div>
                        </div>
                    </div>

                    <div class="flex items-center gap-3 border-t border-border pt-4">
                        <button type="button" :disabled="!canSave" @click="save" class="btn btn-primary">
                            <span v-if="saving"
                                class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                            <span>{{ saving ? $t('common.saving') : $t('common.save') }}</span>
                        </button>
                    </div>
                </div>
            </div>

            <!-- ===== preview ===== -->
            <div class="lg:col-span-5 panel overflow-hidden">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t('settings.preview') }}</span>
                    </span>
                    <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted shrink-0">{{ lateTolMin
                        }} / {{ overtimeTolMin }} {{ $t('settings.minutes') }}</span>
                </div>
                <div class="panel-body">
                    <ul class="space-y-2">
                        <li class="flex items-start gap-2 text-[12.5px] leading-relaxed text-text">
                            <span class="mt-1.5 w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                            {{ $t('settings.previewInOnTime', { min: lateTolMin }) }}
                        </li>
                        <li class="flex items-start gap-2 text-[12.5px] leading-relaxed text-text">
                            <span class="mt-1.5 w-1.5 h-1.5 rounded-full bg-amber-500 shrink-0"></span>
                            {{ $t('settings.previewInLate', { min: lateTolMin + 1 }) }}
                        </li>
                        <li class="flex items-start gap-2 text-[12.5px] leading-relaxed text-text">
                            <span class="mt-1.5 w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                            {{ $t('settings.previewOutOntime', { min1: overtimeTolMin }) }}
                        </li>
                        <li class="flex items-start gap-2 text-[12.5px] leading-relaxed text-text">
                            <span class="mt-1.5 w-1.5 h-1.5 rounded-full bg-amber-500 shrink-0"></span>
                            {{ $t('settings.previewOutEarly', { min1: overtimeTolMin + 1 }) }}
                        </li>
                    </ul>
                </div>
            </div>
        </div>

        <!-- ===== holiday calendar ===== -->
        <PanelHoliday />
    </div>
</template>
