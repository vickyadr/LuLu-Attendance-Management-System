import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from '~/store/auth'

// Toleransi kehadiran, disimpan di backend (tabel `settings`) supaya konsisten
// untuk semua user dan langsung memengaruhi perhitungan report.
export const useSettingsStore = defineStore('settings', () => {
    const auth = useAuthStore()
    const config = useRuntimeConfig()

    const lateTolSec = ref(60)
    const overtimeTolSec = ref(60)
    const loaded = ref(false)

    async function fetchSettings() {
        try {
            const resp = await $fetch(`${config.public.apiBase}/settings/list`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            if (resp.code === 200 && Array.isArray(resp.data)) {
                for (const row of resp.data) {
                    if (row.setting_key === 'late_tolerance_sec') lateTolSec.value = Number(row.setting_value)
                    if (row.setting_key === 'overtime_tolerance_sec') overtimeTolSec.value = Number(row.setting_value)
                }
            }
        } catch (e) {
            console.warn('[settings] load failed', e)
        }
        loaded.value = true
        return { late_tolerance_sec: lateTolSec.value, overtime_tolerance_sec: overtimeTolSec.value }
    }

    async function saveSettings({ late_tolerance_sec, overtime_tolerance_sec }) {
        const resp = await $fetch(`${config.public.apiBase}/settings/edit`, {
            method: 'POST',
            headers: auth.confHeaders(),
            body: { late_tolerance_sec, overtime_tolerance_sec },
        })
        if (resp.code === 200) {
            lateTolSec.value = late_tolerance_sec
            overtimeTolSec.value = overtime_tolerance_sec
        }
        return resp
    }

    return { lateTolSec, overtimeTolSec, loaded, fetchSettings, saveSettings }
})
