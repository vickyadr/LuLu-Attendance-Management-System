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
    const cur = await settings.fetchSettings()
    lateTolMin.value = Math.round((cur.late_tolerance_sec || 0) / 60)
    overtimeTolMin.value = Math.round((cur.overtime_tolerance_sec || 0) / 60)
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
    <div class="space-y-4">
        <!-- Header -->
        <div
            class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col sm:flex-row sm:items-center gap-3 tech-grid-soft shrink-0">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span><span
                class="hud-corner hud-corner-br hidden sm:block"></span>
            <div class="flex items-start gap-3">
                <div
                    class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <font-awesome :icon="faSliders" class="w-[18px] h-[18px]" />
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">{{
                        $t('settings.attendance') }}</h1>
                    <div class="mt-2 flex flex-wrap items-center gap-2 mono text-[10px]">
                        <span class="chip chip-emerald">{{ $t('settings.thresholds') }}</span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Tolerance settings -->
        <div class="hud-frame glass rounded-2xl border border-emerald-100 p-4 sm:p-5 relative overflow-hidden">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span><span
                class="hud-corner hud-corner-br hidden sm:block"></span>

            <div class="space-y-6">
                <!-- Late tolerance -->
                <div>
                    <label for="late-tol"
                        class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">{{
                            $t('settings.lateTolerance') }}</label>
                    <div class="flex items-center gap-2 mt-2">
                        <input id="late-tol" v-model.number="lateTolMin" type="number" min="0" max="60" step="1"
                            class="w-24 h-11 rounded-xl border-2 border-emerald-100 bg-white px-4 text-[15px] font-medium text-green-900 focus:outline-none focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 transition">
                        <span class="mono text-xs text-emerald-700/60">{{ $t('settings.minutes') }}</span>
                    </div>
                </div>

                <!-- Overtime tolerance -->
                <div>
                    <label for="overtime-tol"
                        class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">{{
                            $t('settings.overtimeTolerance') }}</label>
                    <div class="flex items-center gap-2 mt-2">
                        <input id="overtime-tol" v-model.number="overtimeTolMin" type="number" min="0" max="60" step="1"
                            class="w-24 h-11 rounded-xl border-2 border-emerald-100 bg-white px-4 text-[15px] font-medium text-green-900 focus:outline-none focus:border-emerald-400 focus:ring-4 focus:ring-emerald-100 transition">
                        <span class="mono text-xs text-emerald-700/60">{{ $t('settings.minutes') }}</span>
                    </div>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-3 pt-2">
                    <button type="button" :disabled="!canSave" @click="save"
                        class="btn-emerald px-6 h-11 rounded-xl text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed transition">
                        <span v-if="saving"
                            class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                        <span>{{ saving ? $t('common.saving') : $t('common.save') }}</span>
                    </button>

                </div>

                <!-- Preview -->
                <div
                    class="mt-2 rounded-xl bg-emerald-50/60 border border-emerald-100 px-4 py-3 mono text-[11px] leading-relaxed text-emerald-800">
                    <span class="font-bold tracking-widest">{{ $t('settings.preview') }}</span>
                    <p class="mt-1.5">IN at shift start +{{ lateTolMin }} min → on time</p>
                    <p>IN at shift start +{{ lateTolMin + 1 }} min → late</p>
                    <p>OUT at shift end +{{ overtimeTolMin }} min → on time</p>
                    <p>OUT at shift end +{{ overtimeTolMin + 1 }} min → overtime</p>
                </div>
            </div>
        </div>

        <!-- Holiday calendar -->
        <PanelHoliday />
    </div>
</template>
