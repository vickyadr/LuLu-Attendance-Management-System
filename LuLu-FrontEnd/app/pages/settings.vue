<script setup>
import { useSettingsStore } from '~/store/settings'
import { useAuthStore } from '~/store/auth'
import { faSliders } from '@fortawesome/free-solid-svg-icons'

const settings = useSettingsStore()
const auth = useAuthStore()

useHead({ title: 'LuLu — Settings' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

const lateTolMin = ref(1)
const overtimeTolMin = ref(1)
const saving = ref(false)
const saved = ref(false)
const saveError = ref('')
let savedTimer

const canSave = computed(() => !saving.value && lateTolMin.value >= 0 && lateTolMin.value <= 60 && overtimeTolMin.value >= 0 && overtimeTolMin.value <= 60)

// Nilai diambil dari backend dan dikonversi ke menit untuk input; DB menyimpan detik.
onMounted(async () => {
    const cur = await settings.fetchSettings()
    lateTolMin.value = Math.round((cur.late_tolerance_sec || 0) / 60)
    overtimeTolMin.value = Math.round((cur.overtime_tolerance_sec || 0) / 60)
})

async function save() {
    if (!canSave.value) return
    saving.value = true
    saveError.value = ''
    try {
        const resp = await settings.saveSettings({
            late_tolerance_sec: Math.round(lateTolMin.value * 60),
            overtime_tolerance_sec: Math.round(overtimeTolMin.value * 60),
        })
        if (resp.code === 200) {
            saved.value = true
            clearTimeout(savedTimer)
            savedTimer = setTimeout(() => { saved.value = false }, 2000)
        } else {
            saveError.value = resp.message || 'Save failed'
        }
    } catch (e) {
        saveError.value = 'Network error — is the backend running?'
    }
    saving.value = false
}
</script>

<template>
    <div class="space-y-4 max-w-xl">
        <!-- Header -->
        <div
            class="hud-frame glass rounded-[1.5rem] border border-emerald-100 p-6 sm:p-8 relative overflow-hidden tech-pattern">
            <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>
            <div class="pointer-events-none absolute inset-0 tech-dots opacity-[0.04]"></div>
            <div class="flex items-center gap-4">
                <div
                    class="w-12 h-12 rounded-2xl bg-gradient-to-br from-emerald-500 to-green-600 flex items-center justify-center shadow-lg shadow-emerald-200 shrink-0">
                    <font-awesome :icon="faSliders" class="w-6 h-6 text-white" />
                </div>
                <div class="min-w-0">
                    <h2 class="display text-xl font-extrabold text-green-900">Attendance Rules</h2>
                    <p class="mono text-[10px] tracking-[0.14em] text-emerald-700/50 mt-0.5">LATE &amp; OVERTIME
                        THRESHOLDS</p>
                </div>
            </div>
        </div>

        <!-- Tolerance settings -->
        <div class="hud-frame glass rounded-[1.5rem] border border-emerald-100 p-6 sm:p-8 relative overflow-hidden">
            <span class="hud-corner hud-corner-tl"></span><span class="hud-corner hud-corner-br"></span>

            <div class="space-y-6">
                <!-- Late tolerance -->
                <div>
                    <label for="late-tol"
                        class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">Late
                        tolerance</label>
                    <div class="flex items-center gap-2 mt-2">
                        <input id="late-tol" v-model.number="lateTolMin" type="number" min="0" max="60" step="1"
                            class="w-24 h-11 rounded-xl border bg-white px-4 text-[15px] font-medium text-green-900 focus:outline-none border-emerald-200 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10 transition">
                        <span class="mono text-xs text-emerald-700/60">minutes</span>
                    </div>
                </div>

                <!-- Overtime tolerance -->
                <div>
                    <label for="overtime-tol"
                        class="mono text-[11px] font-bold tracking-[0.12em] uppercase text-emerald-800">Overtime
                        tolerance</label>
                    <div class="flex items-center gap-2 mt-2">
                        <input id="overtime-tol" v-model.number="overtimeTolMin" type="number" min="0" max="60" step="1"
                            class="w-24 h-11 rounded-xl border bg-white px-4 text-[15px] font-medium text-green-900 focus:outline-none border-emerald-200 focus:border-emerald-500 focus:ring-4 focus:ring-emerald-500/10 transition">
                        <span class="mono text-xs text-emerald-700/60">minutes</span>
                    </div>
                </div>

                <!-- Actions -->
                <div class="flex items-center gap-3 pt-2">
                    <button type="button" :disabled="!canSave" @click="save"
                        class="btn-emerald px-6 h-11 rounded-xl text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed transition">
                        <span v-if="saving"
                            class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                        <svg v-else-if="saved" xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none"
                            viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                        </svg>
                        <span>{{ saving ? 'Saving…' : (saved ? 'Saved ✓' : 'Save') }}</span>
                    </button>
                    <span v-if="saveError" class="mono text-xs text-red-600">{{ saveError }}</span>
                </div>

                <!-- Preview -->
                <div
                    class="mt-2 rounded-xl bg-emerald-50/60 border border-emerald-100 px-4 py-3 mono text-[11px] leading-relaxed text-emerald-800">
                    <span class="font-bold tracking-widest">PREVIEW</span>
                    <p class="mt-1.5">IN at shift start +{{ lateTolMin }} min → on time</p>
                    <p>IN at shift start +{{ lateTolMin + 1 }} min → late</p>
                    <p>OUT at shift end +{{ overtimeTolMin }} min → on time</p>
                    <p>OUT at shift end +{{ overtimeTolMin + 1 }} min → overtime</p>
                </div>
            </div>
        </div>
    </div>
</template>
