import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from '~/store/auth'

// Leave requests (cuti / izin), owned by the backend (`leave_requests` table).
//
// The approval state lives here only as a mirror: `pending` requests are visible
// in this panel but touch no attendance, and only an `approved` row removes a
// person's day from the report. The report engine reads the table directly, so
// approving here changes the next report query with no client cache to flush.
export const useLeaveStore = defineStore('leave', () => {
    const auth = useAuthStore()
    const config = useRuntimeConfig()

    const items = ref([])
    const employees = ref([])
    const employeesLoaded = ref(false)
    const loaded = ref(false)
    const saving = ref(false)

    async function fetchLeaves({ from, to, employeeId } = {}) {
        try {
            const params = new URLSearchParams()
            if (from) params.set('from', from)
            if (to) params.set('to', to)
            if (employeeId) params.set('employee_id', String(employeeId))
            const qs = params.toString()
            const resp = await apiFetch(`${config.public.apiBase}/leave/list${qs ? `?${qs}` : ''}`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            if (resp.code === 200 && Array.isArray(resp.data)) items.value = resp.data
        } catch (e) {
            console.warn('[leave] load failed', e)
        }
        loaded.value = true
        return items.value
    }

    // Create or correct a request. The backend upserts on (employee, date), so
    // re-submitting the same day edits the existing request and returns it to
    // `pending` rather than filing a duplicate.
    async function saveLeave({ leave_employee_id, leave_date, leave_kind, leave_reason }) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/leave/edit`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { leave_employee_id, leave_date, leave_kind, leave_reason },
            })
            if (resp.code === 200) await fetchLeaves()
            return resp
        } finally {
            saving.value = false
        }
    }

    // Approve or reject. Rejecting keeps the row as a decision record; the
    // report ignores it, so a rejected request never changes attendance.
    async function decide(leave_id, approve) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/leave/decision`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { leave_id, leave_approve: !!approve },
            })
            if (resp.code === 200) await fetchLeaves()
            return resp
        } finally {
            saving.value = false
        }
    }

    // Withdraw a request outright: the person changed their mind or the date
    // was entered by mistake. This is a delete, not a status.
    async function cancelLeave(leave_id) {
        saving.value = true
        try {
            const resp = await apiFetch(`${config.public.apiBase}/leave/cancel`, {
                method: 'POST',
                headers: auth.confHeaders(),
                body: { leave_id },
            })
            if (resp.code === 200) await fetchLeaves()
            return resp
        } finally {
            saving.value = false
        }
    }

    // Minimal name picker for the request form, so this page never needs the
    // full employee page (which carries addresses and other personal columns an
    // attendance administrator has no reason to read).
    async function fetchEmployees() {
        try {
            const resp = await apiFetch(`${config.public.apiBase}/leave/employee-options`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            if (resp.code === 200 && Array.isArray(resp.data)) employees.value = resp.data
        } catch (e) {
            console.warn('[leave] employees failed', e)
        }
        employeesLoaded.value = true
        return employees.value
    }

    return { items, employees, employeesLoaded, loaded, saving, fetchLeaves, fetchEmployees, saveLeave, decide, cancelLeave }
})