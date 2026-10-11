import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useAuthStore } from '~/store/auth'

// Calendar of non-working days, owned by the backend (`holiday` table) so every
// user and every report query reads the same rows.
//
// The report engine reads this table directly, so a day saved here stops
// producing a report row the moment the backend picks it up - there is no
// client-side cache to invalidate and no second source of truth to drift.
export const useHolidayStore = defineStore('holiday', () => {
    const auth = useAuthStore()
    const config = useRuntimeConfig()

    const items = ref([])
    const loaded = ref(false)
    const saving = ref(false)

    // Newest first for editing, oldest first for the calendar strip.
    const sortedByDate = computed(() =>
        [...items.value].sort((a, b) => a.holiday_date.localeCompare(b.holiday_date))
    )

    function byKind(kind) {
        return sortedByDate.value.filter((h) => h.holiday_kind === kind)
    }

    async function fetchHolidays({ from, to } = {}) {
        try {
            const params = new URLSearchParams()
            if (from) params.set('from', from)
            if (to) params.set('to', to)
            const qs = params.toString()
            const resp = await apiFetch(`${config.public.apiBase}/holiday/list${qs ? `?${qs}` : ''}`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            if (resp.code === 200 && Array.isArray(resp.data)) {
                items.value = resp.data
            }
        } catch (e) {
            console.warn('[holiday] load failed', e)
        }
        loaded.value = true
        return items.value
    }

    // holiday_included is sent only when the caller decides it. Omitting it (null)
    // tells the backend to keep whatever is stored for that date, which is what an
    // edit of the name or kind alone must do. The list is re-read from the server
    // instead of being patched locally: the toggle drives payroll, so a local guess
    // that disagrees with the stored row would be worse than an extra round trip.
    async function saveHoliday({ holiday_date, holiday_name, holiday_kind, holiday_included = null }) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/holiday/edit`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { holiday_date, holiday_name, holiday_kind, holiday_source: 'manual', holiday_included },
            })
            if (resp.code === 200) await fetchHolidays()
            return resp
        } finally {
            saving.value = false
        }
    }

    async function deleteHoliday(holiday_date) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/holiday/delete`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { holiday_date },
            })
            if (resp.code === 200) {
                items.value = items.value.filter((h) => h.holiday_date !== holiday_date)
            }
            return resp
        } finally {
            saving.value = false
        }
    }

    // Bulk upsert of rows parsed from an imported calendar file. The backend
    // rejects the whole batch if any row is malformed, so a partial calendar is
    // never written.
    async function importHolidays(rows) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/holiday/import`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { holidays: rows },
            })
            if (resp.code === 200) await fetchHolidays()
            return resp
        } finally {
            saving.value = false
        }
    }

    return {
        items,
        loaded,
        saving,
        sortedByDate,
        byKind,
        fetchHolidays,
        saveHoliday,
        deleteHoliday,
        importHolidays,
    }
})
