<script setup>
import { useFieldStore } from '~/store/field'
import { useAuthStore } from '~/store/auth'
import { useNotifyStore } from '~/store/notify'
import { faInbox, faRotate, faMagnifyingGlass } from '@fortawesome/free-solid-svg-icons'

useHead({ title: 'LuLu - Field Attendance' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

const field = useFieldStore()
const auth = useAuthStore()
const notify = useNotifyStore()
const { t } = useI18n()

const tab = ref('review')
const status = ref('pending')
const detail = ref(null)
const rejectReason = ref('')
const showReject = ref(false)
const busy = ref(false)

const accountForm = ref(null) // { employee_id, name, nickname }
const passwordBox = ref(null) // { nickname, password, title }

const canEdit = computed(() => auth.hasActionAccess('field', 'can_edit'))
const canProvision = computed(() => auth.hasActionAccess('users', 'can_add'))
const canReset = computed(() => auth.hasActionAccess('users', 'can_edit'))

const thumbs = ref({})

function fmtDate(value) {
    if (!value) return '-'
    const [y, m, d] = String(value).slice(0, 10).split('-')
    return d ? `${d}/${m}/${y}` : value
}

function fmtTime(value, tz) {
    if (!value) return '-'
    const date = new Date(value)
    if (Number.isNaN(date.getTime())) return '-'
    // The offset stored on the event is the phone's, so the photo time reads the
    // way the employee saw it even when they crossed a timezone.
    if (typeof tz === 'number') {
        const shifted = new Date(date.getTime() + tz * 60000)
        return shifted.toISOString().slice(11, 16)
    }
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function kindLabel(kind) {
    return kind === 'out' ? t('field.kindOut') : t('field.kindIn')
}

function photoOf(day, kind) {
    return (day.photos || []).find((p) => p.kind === kind) || null
}

function completeness(day) {
    const hasIn = Boolean(day.punch_in || photoOf(day, 'in'))
    const hasOut = Boolean(day.punch_out || photoOf(day, 'out'))
    if (hasIn && hasOut) return { key: 'field.complete', tone: 'text-emerald-700 bg-emerald-50 border-emerald-200' }
    if (hasIn || hasOut) return { key: 'field.incomplete', tone: 'text-amber-700 bg-amber-50 border-amber-200' }
    return { key: 'field.empty', tone: 'text-rose-700 bg-rose-50 border-rose-200' }
}

function statusTone(value) {
    if (value === 'approved') return 'text-emerald-700 bg-emerald-50 border-emerald-200'
    if (value === 'rejected') return 'text-rose-700 bg-rose-50 border-rose-200'
    return 'text-amber-700 bg-amber-50 border-amber-200'
}

async function loadThumbs() {
    for (const day of field.days) {
        for (const photo of day.photos || []) {
            if (!thumbs.value[photo.id]) thumbs.value[photo.id] = await field.photoSrc(photo.id)
        }
    }
}

async function load(statusValue = status.value) {
    field.currentStatus = statusValue
    const resp = await field.fetchDays(statusValue === 'all' ? '' : statusValue)
    if (resp.code !== 200) notify.error(t('field.loadFailed'), resp.message)
    await loadThumbs()
}

const accountSearch = ref('')
const showAccountSuggest = ref(false)
let searchTimer = null

const accountSuggestions = computed(() => {
    const term = accountSearch.value.trim().toLowerCase()
    if (!term) return { names: [], departments: [] }
    const names = new Set()
    const departments = new Set()
    for (const account of field.accounts) {
        if (account.employee_name?.toLowerCase().includes(term)) names.add(account.employee_name)
        if (account.department?.toLowerCase().includes(term)) departments.add(account.department)
    }
    return { names: [...names].slice(0, 6), departments: [...departments].slice(0, 4) }
})

function onAccountSearch() {
    showAccountSuggest.value = true
    clearTimeout(searchTimer)
    searchTimer = setTimeout(() => loadAccounts(), 300)
}

function pickSuggestion(value) {
    accountSearch.value = value
    showAccountSuggest.value = false
    loadAccounts()
}

async function loadAccounts(append = false) {
    const resp = await field.fetchAccounts({ append, q: accountSearch.value.trim(), offset: append ? field.accounts.length : 0 })
    if (resp.code !== 200) notify.error(t('field.loadFailed'), resp.message)
}

function setStatus(value) {
    status.value = value
    load(value)
}

function openDetail(day) {
    detail.value = day
    showReject.value = false
    rejectReason.value = ''
}

function closeDetail() {
    detail.value = null
    showReject.value = false
    rejectReason.value = ''
}

async function approve(day) {
    busy.value = true
    try {
        const resp = await field.review(day.id, 'approve')
        if (resp.code === 200) {
            notify.success(t('field.approvedToast'), day.employee_name)
            closeDetail()
            await loadThumbs()
        } else {
            notify.error(t('field.reviewFailed'), resp.message)
        }
    } finally {
        busy.value = false
    }
}

async function reject(day) {
    const reason = rejectReason.value.trim()
    if (!reason) {
        notify.warning(t('field.reasonRequired'))
        return
    }
    busy.value = true
    try {
        const resp = await field.review(day.id, 'reject', reason)
        if (resp.code === 200) {
            notify.success(t('field.rejectedToast'), day.employee_name)
            closeDetail()
            await loadThumbs()
        } else {
            notify.error(t('field.reviewFailed'), resp.message)
        }
    } finally {
        busy.value = false
    }
}

function openCreate(account) {
    accountForm.value = {
        employee_id: account.employee_id,
        name: account.employee_name,
        nickname: suggestedNickname(account),
    }
}

function suggestedNickname(account) {
    const base = String(account.employee_name || '')
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '.')
        .replace(/^\.|\.$/g, '')
        .slice(0, 24)
    return base || `emp${account.employee_id}`
}

async function submitCreate() {
    const form = accountForm.value
    if (!form) return
    const resp = await field.createAccount(form.employee_id, form.nickname.trim())
    if (resp.code === 200) {
        accountForm.value = null
        passwordBox.value = { ...resp.data[0], title: t('field.passwordTitleCreate'), hint: t('field.passwordOnce') }
    } else {
        notify.error(t('field.accountFailed'), resp.message)
    }
}

async function resetPassword(account) {
    const resp = await field.resetPassword(account.employee_id)
    if (resp.code === 200) {
        passwordBox.value = { ...resp.data[0], title: t('field.passwordTitleReset'), hint: t('field.passwordOnce') }
    } else {
        notify.error(t('field.accountFailed'), resp.message)
    }
}

async function copyPassword() {
    if (!passwordBox.value?.password) return
    try {
        await navigator.clipboard.writeText(passwordBox.value.password)
        notify.success(t('field.passwordCopied'))
    } catch {
        notify.warning(t('field.passwordCopyFailed'))
    }
}

function onEsc(event) {
    if (event.key !== 'Escape') return
    if (passwordBox.value) passwordBox.value = null
    else if (accountForm.value) accountForm.value = null
    else if (detail.value) closeDetail()
}

watch([detail, accountForm, passwordBox], async () => {
    await nextTick()
    const dialog = document.querySelector('[data-field-modal]')
    const target = dialog?.querySelector('input, textarea, button')
    target?.focus?.()
})

onMounted(async () => {
    window.addEventListener('keydown', onEsc)
    await Promise.all([load('pending'), loadAccounts()])
})
onUnmounted(() => window.removeEventListener('keydown', onEsc))
</script>

<template>
    <div class="flex flex-col gap-4">

        <!-- ===== page head ===== -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-3">
            <div class="min-w-0">
                <h1 class="display text-[20px] text-text">{{ $t('field.title') }}</h1>
                <p class="mt-1 text-[12.5px] leading-relaxed text-text-muted">
                    {{ tab === 'review' ? $t('field.subtitleReview') : $t('field.subtitleAccounts') }}
                </p>
            </div>
            <div class="sm:ml-auto flex items-center gap-2 shrink-0">
                <button class="btn" :class="tab === 'review' ? 'btn-primary' : 'btn-secondary'" @click="tab = 'review'">
                    {{ $t('field.tabReview') }}
                </button>
                <button class="btn" :class="tab === 'accounts' ? 'btn-primary' : 'btn-secondary'"
                    @click="tab = 'accounts'">
                    {{ $t('field.tabAccounts') }}
                </button>
            </div>
        </div>

        <template v-if="tab === 'review'">
            <!-- ===== controls ===== -->
            <div class="panel p-3 sm:p-4 flex flex-wrap items-center gap-2">
                <button v-for="chip in ['pending', 'approved', 'rejected', 'all']" :key="chip"
                    class="btn btn-sm !min-h-[44px] sm:!min-h-[36px]"
                    :class="status === chip ? 'btn-primary' : 'btn-secondary'" @click="setStatus(chip)">
                    {{ $t(`field.chip_${chip}`) }}
                </button>
                <button class="ml-auto btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]" @click="load()">
                    <font-awesome :icon="faRotate" class="w-3.5 h-3.5" :class="field.loading ? 'animate-spin' : ''" />
                    {{ $t('field.refresh') }}
                </button>
            </div>

            <!-- ===== review queue ===== -->
            <div class="panel overflow-hidden">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full shrink-0"
                            :class="field.loading ? 'bg-amber-500 data-tick' : 'bg-primary'"></span>
                        <span class="truncate">{{ $t('field.tabReview') }}</span>
                    </span>
                    <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted shrink-0">{{
                        field.days.length }}</span>
                </div>

                <div v-if="field.loading" class="p-4 space-y-2.5">
                    <div v-for="n in 3" :key="`sk-${n}`" class="skeleton h-16"></div>
                </div>

                <div v-else-if="field.days.length === 0" class="state">
                    <div class="state-icon">
                        <font-awesome :icon="faInbox" class="w-5 h-5" />
                    </div>
                    <p class="state-title">{{ $t('field.emptyReview') }}</p>
                    <p class="state-hint">{{ $t('field.emptyReviewHint') }}</p>
                </div>

                <div v-else class="divide-y divide-border">
                    <article v-for="day in field.days" :key="day.id" class="p-3 sm:p-4">
                        <div class="flex flex-col lg:flex-row lg:items-start gap-3">
                            <div class="min-w-0 flex-1">
                                <div class="flex flex-wrap items-center gap-1.5">
                                    <p class="text-[13.5px] font-bold text-text truncate">{{ day.employee_name }}</p>
                                    <span class="chip"
                                        :class="day.status === 'approved' ? 'chip-ok' : day.status === 'rejected' ? 'chip-danger' : 'chip-warn'">
                                        {{ $t(`field.status_${day.status}`) }}
                                    </span>
                                    <span class="chip"
                                        :class="completeness(day).key === 'field.complete' ? 'chip-ok' : completeness(day).key === 'field.incomplete' ? 'chip-warn' : 'chip-danger'">
                                        {{ $t(completeness(day).key) }}
                                    </span>
                                </div>
                                <p class="mt-1.5 mono text-[11px] text-text-muted">
                                    {{ fmtDate(day.work_date) }}
                                    <span class="mx-1 text-border-strong">/</span>
                                    {{ $t('field.deviceIn') }} {{ day.punch_in ? fmtTime(day.punch_in) : '-' }}
                                    <span class="mx-1 text-border-strong">/</span>
                                    {{ $t('field.deviceOut') }} {{ day.punch_out ? fmtTime(day.punch_out) : '-' }}
                                </p>
                                <p v-if="day.note" class="mt-1.5 text-[12.5px] leading-relaxed text-text">{{ day.note }}
                                </p>
                            </div>

                            <div class="flex items-end gap-2 shrink-0">
                                <div v-for="photo in day.photos" :key="photo.id">
                                    <img v-if="thumbs[photo.id]" :src="thumbs[photo.id]" :alt="kindLabel(photo.kind)"
                                        class="w-16 h-16 object-cover rounded-lg border border-border" />
                                    <div v-else class="w-16 h-16 rounded-lg skeleton"></div>
                                    <p
                                        class="mt-1 text-center mono text-[10px] tracking-[0.08em] uppercase text-text-muted">
                                        {{ kindLabel(photo.kind) }}</p>
                                </div>
                            </div>
                        </div>

                        <div class="mt-3 flex flex-wrap items-center gap-2">
                            <button class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                                @click="openDetail(day)">{{ $t('field.detail') }}</button>
                            <template v-if="canEdit && day.status !== 'approved'">
                                <button class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]" :disabled="busy"
                                    @click="approve(day)">{{ $t('field.approve') }}</button>
                            </template>
                            <template v-if="canEdit && day.status !== 'rejected'">
                                <button class="btn btn-danger btn-sm !min-h-[44px] sm:!min-h-[36px]" :disabled="busy"
                                    @click="openDetail(day); showReject = true">{{ $t('field.reject') }}</button>
                            </template>
                        </div>
                    </article>
                </div>
            </div>
        </template>

        <template v-else>
            <!-- ===== employee accounts ===== -->
            <div class="panel overflow-hidden">
                <div class="panel-head">
                    <span class="inline-flex items-center gap-2 min-w-0">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                        <span class="truncate">{{ $t('field.accountsTitle') }}</span>
                    </span>
                    <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted shrink-0">
                        {{ field.accountsHasMore ? $t('field.accountsCountMore', { n: field.accounts.length }) :
                            $t('field.accountsCount', { n: field.accounts.length }) }}
                    </span>
                </div>

                <div class="relative p-3 border-b border-border">
                    <div class="relative">
                        <font-awesome :icon="faMagnifyingGlass"
                            class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
                        <input v-model="accountSearch" type="search" :placeholder="$t('field.accountSearch')"
                            class="field has-icon-left" @input="onAccountSearch"
                            @focus="showAccountSuggest = !!accountSearch.trim()" @blur="showAccountSuggest = false"
                            @keydown.esc="showAccountSuggest = false" />
                    </div>
                    <div v-if="showAccountSuggest && (accountSuggestions.names.length || accountSuggestions.departments.length)"
                        class="absolute left-0 right-0 top-full z-40 mt-1 max-h-64 overflow-y-auto rounded-xl border border-border bg-surface shadow-lg">
                        <template v-if="accountSuggestions.names.length">
                            <p class="px-3 py-1.5 card-label bg-surface-raised border-b border-border">{{
                                $t('field.suggestName') }}</p>
                            <button v-for="name in accountSuggestions.names" :key="'n-' + name" type="button"
                                class="block w-full px-3 py-2.5 min-h-[44px] text-left text-[13px] text-text hover:bg-surface-raised"
                                @mousedown.prevent="pickSuggestion(name)">{{ name }}</button>
                        </template>
                        <template v-if="accountSuggestions.departments.length">
                            <p class="px-3 py-1.5 card-label bg-surface-raised border-b border-border">{{
                                $t('field.suggestDepartment') }}</p>
                            <button v-for="dept in accountSuggestions.departments" :key="'d-' + dept" type="button"
                                class="block w-full px-3 py-2.5 min-h-[44px] text-left text-[13px] text-text hover:bg-surface-raised"
                                @mousedown.prevent="pickSuggestion(dept)">{{ dept }}</button>
                        </template>
                    </div>
                </div>

                <div v-if="field.accountsLoading && !field.accounts.length" class="p-4 space-y-2">
                    <div v-for="n in 4" :key="`ask-${n}`" class="skeleton h-11"></div>
                </div>

                <div v-else-if="!field.accounts.length" class="state">
                    <div class="state-icon">
                        <font-awesome :icon="faInbox" class="w-5 h-5" />
                    </div>
                    <p class="state-title">{{ $t('field.accountsNoMatch') }}</p>
                    <p class="state-hint">{{ $t('field.subtitleAccounts') }}</p>
                </div>

                <template v-else>
                    <table class="w-full border-collapse hidden md:table">
                        <thead>
                            <tr>
                                <th class="th">{{ $t('employee.colEmployee') }}</th>
                                <th class="th">{{ $t('field.suggestDepartment') }}</th>
                                <th class="th">{{ $t('user.colNickname') }}</th>
                                <th class="th !text-right">{{ $t('common.actions') }}</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="account in field.accounts" :key="account.employee_id"
                                class="even:bg-surface-raised/25">
                                <td class="td">
                                    <p class="text-[13px] font-semibold text-text truncate">{{ account.employee_name }}
                                    </p>
                                    <p class="mono text-[10px] text-text-muted mt-0.5">#{{ account.employee_id }}</p>
                                </td>
                                <td class="td">{{ account.department || '-' }}</td>
                                <td class="td">
                                    <span v-if="account.nickname" class="mono text-[11.5px] text-text">{{
                                        account.nickname }}</span>
                                    <span v-else class="text-[12.5px] text-text-muted">{{ $t('field.noAccount')
                                        }}</span>
                                </td>
                                <td class="td !text-right">
                                    <button v-if="!account.has_user && canProvision"
                                        class="btn btn-primary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                                        @click="openCreate(account)">{{ $t('field.createAccount') }}</button>
                                    <button v-else-if="account.has_user && canReset"
                                        class="btn btn-secondary btn-sm !min-h-[44px] sm:!min-h-[36px]"
                                        @click="resetPassword(account)">{{ $t('field.resetPassword') }}</button>
                                </td>
                            </tr>
                        </tbody>
                    </table>

                    <div class="md:hidden divide-y divide-border">
                        <div v-for="account in field.accounts" :key="`m-${account.employee_id}`" class="p-3">
                            <div class="flex items-start justify-between gap-3">
                                <div class="min-w-0">
                                    <p class="text-[13px] font-semibold text-text truncate">{{ account.employee_name }}
                                    </p>
                                    <p class="mono text-[10px] text-text-muted mt-0.5">
                                        #{{ account.employee_id }}
                                        <span class="mx-1 text-border-strong">/</span>
                                        {{ account.department || '-' }}
                                    </p>
                                </div>
                                <span v-if="account.nickname" class="chip chip-quiet shrink-0">{{ account.nickname
                                    }}</span>
                                <span v-else class="chip chip-warn shrink-0">{{ $t('field.noAccount') }}</span>
                            </div>
                            <div class="mt-2">
                                <button v-if="!account.has_user && canProvision"
                                    class="btn btn-primary btn-sm !min-h-[44px] w-full" @click="openCreate(account)">{{
                                    $t('field.createAccount') }}</button>
                                <button v-else-if="account.has_user && canReset"
                                    class="btn btn-secondary btn-sm !min-h-[44px] w-full"
                                    @click="resetPassword(account)">{{ $t('field.resetPassword') }}</button>
                            </div>
                        </div>
                    </div>
                </template>

                <div v-if="field.accountsHasMore" class="p-3 border-t border-border">
                    <button class="btn btn-secondary w-full" :disabled="field.accountsLoading"
                        @click="loadAccounts(true)">
                        {{ field.accountsLoading ? $t('common.loading') : $t('field.accountsLoadMore') }}
                    </button>
                </div>
            </div>
        </template>

        <!-- ===== day detail ===== -->
        <div v-if="detail"
            class="fixed inset-0 z-[100] bg-black/40 flex items-end md:items-center justify-center p-0 md:p-6"
            @click.self="closeDetail">
            <div data-field-modal role="dialog" aria-modal="true" aria-labelledby="field-detail-title"
                class="w-full md:max-w-3xl max-h-[92vh] overflow-y-auto panel rounded-t-2xl md:rounded-2xl">
                <div
                    class="sticky top-0 z-10 bg-surface px-4 py-3 border-b border-border flex items-start justify-between gap-3">
                    <div class="min-w-0">
                        <p id="field-detail-title" class="text-[14px] font-bold text-text truncate">{{
                            detail.employee_name }}
                        </p>
                        <p class="mono text-[11px] text-text-muted mt-0.5">{{ fmtDate(detail.work_date) }}</p>
                    </div>
                    <button class="btn btn-quiet btn-sm !min-h-[44px] shrink-0" @click="closeDetail">{{
                        $t('common.close')
                        }}</button>
                </div>

                <div class="p-4 space-y-4">
                    <div class="flex flex-wrap items-center gap-1.5">
                        <span class="chip"
                            :class="detail.status === 'approved' ? 'chip-ok' : detail.status === 'rejected' ? 'chip-danger' : 'chip-warn'">
                            {{ $t(`field.status_${detail.status}`) }}
                        </span>
                        <span class="chip"
                            :class="completeness(detail).key === 'field.complete' ? 'chip-ok' : completeness(detail).key === 'field.incomplete' ? 'chip-warn' : 'chip-danger'">
                            {{ $t(completeness(detail).key) }}
                        </span>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                        <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
                            <p class="mono text-[10px] tracking-[0.1em] uppercase text-text-muted">{{
                                $t('field.deviceIn') }}
                            </p>
                            <p class="mono text-[13px] font-bold text-text mt-0.5">{{ detail.punch_in ?
                                fmtTime(detail.punch_in)
                                : '-' }}</p>
                        </div>
                        <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
                            <p class="mono text-[10px] tracking-[0.1em] uppercase text-text-muted">{{
                                $t('field.deviceOut') }}
                            </p>
                            <p class="mono text-[13px] font-bold text-text mt-0.5">{{ detail.punch_out ?
                                fmtTime(detail.punch_out) : '-' }}</p>
                        </div>
                    </div>

                    <div class="grid gap-3 sm:grid-cols-2">
                        <div v-for="photo in detail.photos" :key="photo.id"
                            class="rounded-xl border border-border overflow-hidden bg-surface">
                            <img v-if="thumbs[photo.id]" :src="thumbs[photo.id]" :alt="kindLabel(photo.kind)"
                                class="w-full h-56 object-cover" />
                            <div v-else class="w-full h-56 skeleton"></div>
                            <div class="p-2.5 text-[11px] text-text space-y-0.5">
                                <p class="mono text-[10px] tracking-[0.08em] uppercase text-text-muted font-bold">{{
                                    kindLabel(photo.kind) }}</p>
                                <p>
                                    {{ $t('field.takenAt') }} <span class="mono">{{ fmtTime(photo.server_at,
                                        photo.tz_offset_minutes) }}</span>
                                    <span class="text-text-muted mono">({{ photo.tz_offset_minutes >= 0 ? '+' : '' }}{{
                                        (photo.tz_offset_minutes || 0) / 60 }}h)</span>
                                </p>
                                <p>{{ $t('field.deviceTime') }} <span class="mono">{{ photo.device_at ?
                                    fmtTime(photo.device_at,
                                        photo.tz_offset_minutes) : '-' }}</span></p>
                                <p>{{ $t('field.accuracy') }} <span class="mono">{{ photo.accuracy_m != null ?
                                    `${Math.round(photo.accuracy_m)} m` : '-' }}</span></p>
                                <p v-if="photo.lat != null">
                                    <a class="underline underline-offset-2 mono" target="_blank" rel="noopener"
                                        :href="`https://www.openstreetmap.org/?mlat=${photo.lat}&mlon=${photo.lng}#map=16/${photo.lat}/${photo.lng}`">
                                        {{ photo.lat.toFixed(5) }}, {{ photo.lng.toFixed(5) }}
                                    </a>
                                </p>
                            </div>
                        </div>
                    </div>

                    <p v-if="detail.note" class="text-[13px] leading-relaxed text-text">{{ detail.note }}</p>
                    <p v-if="detail.review_reason"
                        class="rounded-lg bg-danger-bg border border-red-200 px-3 py-2 text-[12.5px] leading-relaxed text-danger">
                        {{ $t('field.reasonLabel') }}: {{ detail.review_reason }}
                    </p>

                    <div v-if="canEdit" class="space-y-2 border-t border-border pt-4">
                        <label class="label" for="field-reject-reason">{{ $t('field.reasonLabel') }}</label>
                        <textarea id="field-reject-reason" v-model="rejectReason" rows="2" class="field"
                            :placeholder="$t('field.reasonPlaceholder')"></textarea>
                        <div class="flex flex-wrap gap-2">
                            <button class="btn btn-primary" :disabled="busy" @click="approve(detail)">{{
                                $t('field.approve')
                                }}</button>
                            <button class="btn btn-danger" :disabled="busy" @click="reject(detail)">{{
                                $t('field.reject')
                                }}</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- ===== create account ===== -->
        <div v-if="accountForm" class="fixed inset-0 z-[100] bg-black/40 flex items-center justify-center p-4"
            @click.self="accountForm = null">
            <div data-field-modal role="dialog" aria-modal="true" aria-labelledby="field-account-title"
                class="w-full max-w-md panel overflow-hidden">
                <div class="panel-head">
                    <span id="field-account-title" class="truncate">{{ $t('field.createAccount') }}</span>
                </div>
                <div class="panel-body space-y-3">
                    <p class="text-[13.5px] font-bold text-text">{{ accountForm.name }}</p>
                    <div>
                        <label class="label" for="field-nickname">{{ $t('field.nickname') }}</label>
                        <input id="field-nickname" v-model="accountForm.nickname" maxlength="30" class="field mono" />
                        <p class="mt-1 text-[11.5px] text-text-muted">{{ $t('field.nicknameHint') }}</p>
                    </div>
                    <div class="flex justify-end gap-2 pt-1">
                        <button class="btn btn-secondary" @click="accountForm = null">{{ $t('common.cancel') }}</button>
                        <button class="btn btn-primary" @click="submitCreate">{{ $t('field.generatePassword')
                            }}</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- ===== password shown once ===== -->
        <div v-if="passwordBox" class="fixed inset-0 z-[100] bg-black/40 flex items-center justify-center p-4"
            @click.self="passwordBox = null">
            <div data-field-modal role="dialog" aria-modal="true" aria-labelledby="field-password-title"
                class="w-full max-w-md panel overflow-hidden">
                <div class="panel-head">
                    <span id="field-password-title" class="truncate">{{ passwordBox.title }}</span>
                </div>
                <div class="panel-body space-y-3">
                    <p class="mono text-[11px] text-text-muted">{{ passwordBox.nickname }}</p>
                    <div class="rounded-xl bg-surface-raised border border-border px-3 py-3 text-center">
                        <p class="mono text-[20px] font-bold tracking-[0.08em] text-text select-all">{{
                            passwordBox.password }}
                        </p>
                    </div>
                    <p class="text-[11.5px] leading-relaxed text-warn">{{ passwordBox.hint }}</p>
                    <div class="flex justify-end gap-2">
                        <button class="btn btn-secondary" @click="copyPassword">{{ $t('field.copy') }}</button>
                        <button class="btn btn-primary" @click="passwordBox = null">{{ $t('common.close') }}</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
