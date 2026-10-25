import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from '~/store/auth'

// Field attendance admin side: approve/reject a day, and provision the employee
// logins the PWA uses. A day is one employee + one office-timezone date; the
// device punch and the field photos are shown together because the admin
// decides the whole day, not each photo.
export const useFieldStore = defineStore('field', () => {
    const auth = useAuthStore()
    const config = useRuntimeConfig()

    const days = ref([])
    const accounts = ref([])
    const accountQuery = ref('')
    const accountsHasMore = ref(false)
    const loading = ref(false)
    const accountsLoading = ref(false)
    const photoCache = new Map()

    async function fetchDays(status = 'pending') {
        loading.value = true
        try {
            const params = new URLSearchParams()
            if (status) params.set('status', status)
            const qs = params.toString()
            const resp = await apiFetch(`${config.public.apiBase}/field/attendances${qs ? `?${qs}` : ''}`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            days.value = resp.code === 200 && Array.isArray(resp.data) ? resp.data : []
            return resp
        } finally {
            loading.value = false
        }
    }

    async function review(id, action, reason = '') {
        const resp = await apiFetch(`${config.public.apiBase}/field/attendances/${id}/review`, {
            method: 'PUT',
            headers: auth.confHeaders(),
            body: { action, reason },
        })
        if (resp.code === 200) await fetchDays(currentStatus.value)
        return resp
    }

    const currentStatus = ref('pending')

    // One page at a time keeps the tab light on a large employee table.
    const ACCOUNT_PAGE = 50

    async function fetchAccounts({ append = false, q = accountQuery.value, offset = 0 } = {}) {
        accountsLoading.value = true
        try {
            const params = new URLSearchParams({ limit: String(ACCOUNT_PAGE), offset: String(offset) })
            if (q) params.set('q', q)
            const resp = await apiFetch(`${config.public.apiBase}/field/accounts?${params}`, {
                method: 'GET',
                headers: auth.confHeaders(),
            })
            const payload = resp.code === 200 && Array.isArray(resp.data)
                ? (Array.isArray(resp.data[0]?.items) ? resp.data[0] : { items: resp.data })
                : { items: [] }
            const rows = payload.items || []
            accounts.value = append ? [...accounts.value, ...rows] : rows
            accountsHasMore.value = accounts.value.length < (payload.total ?? rows.length)
            accountQuery.value = q
            return resp
        } finally {
            accountsLoading.value = false
        }
    }

    async function createAccount(employeeId, nickname) {
        const resp = await apiFetch(`${config.public.apiBase}/field/accounts`, {
            method: 'POST',
            headers: auth.confHeaders(),
            body: { employee_id: employeeId, nickname },
        })
        if (resp.code === 200) await fetchAccounts()
        return resp
    }

    async function resetPassword(employeeId) {
        const resp = await apiFetch(`${config.public.apiBase}/field/accounts/${employeeId}/reset`, {
            method: 'POST',
            headers: auth.confHeaders(),
            body: {},
        })
        if (resp.code === 200) await fetchAccounts()
        return resp
    }

    // Photos sit behind the auth header, so an <img src> cannot fetch them.
    async function photoSrc(id) {
        if (photoCache.has(id)) return photoCache.get(id)
        const resp = await fetch(`${config.public.apiBase}/field/photos/${id}`, {
            headers: { Authorization: auth.confHeaders().Authorization },
        })
        if (!resp.ok) return ''
        const url = URL.createObjectURL(await resp.blob())
        photoCache.set(id, url)
        return url
    }

    return {
        days, accounts, accountQuery, accountsHasMore, loading, accountsLoading, currentStatus,
        fetchDays, review, fetchAccounts, createAccount, resetPassword, photoSrc,
    }
})
