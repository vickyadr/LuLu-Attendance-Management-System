<script setup>
import { useLeaveStore } from '~/store/leave'
import { useAuthStore } from '~/store/auth'
import { useNotifyStore } from '~/store/notify'
import { faCalendarPlus, faTrash } from '@fortawesome/free-solid-svg-icons'

useHead({ title: 'LuLu - Leave' })
definePageMeta({ middleware: ['get-auth'], layout: 'default' })

const leave = useLeaveStore()
const notify = useNotifyStore()
const { t } = useI18n()

const KINDS = [
    { value: 'cuti', label: 'leave.kindCuti' },
    { value: 'izin', label: 'leave.kindIzin' },
    { value: 'sakit', label: 'leave.kindSakit' },
]

const empQuery = ref('')
const statusFilter = ref('')
const chosenId = ref(0)
const newDate = ref('')
const newKind = ref('izin')
const newReason = ref('')

const canSubmit = computed(() => chosenId.value > 0 && /^\d{4}-\d{2}-\d{2}$/.test(newDate.value))

const employeeOptions = computed(() => {
    const q = empQuery.value.trim().toLowerCase()
    if (!q) return leave.employees
    return leave.employees.filter((e) =>
        `${e.first_name || ''} ${e.last_name || ''} ${e.departement || ''}`.toLowerCase().includes(q)
    )
})

const chosen = computed(() => leave.employees.find((e) => e.id === chosenId.value))

const visible = computed(() => {
    let rows = leave.items
    if (chosenId.value) rows = rows.filter((r) => r.leave_employee_id === chosenId.value)
    else if (empQuery.value.trim()) {
        const q = empQuery.value.trim().toLowerCase()
        rows = rows.filter((r) =>
            `${r.employee_fname || ''} ${r.employee_lname || ''}`.toLowerCase().includes(q)
        )
    }
    if (statusFilter.value) rows = rows.filter((r) => r.leave_status === statusFilter.value)
    return rows
})

function displayName(r) {
    return `${r.employee_fname || ''} ${r.employee_lname || ''}`.trim() || `#${r.leave_employee_id}`
}

function kindLabel(kind) {
    return KINDS.find((k) => k.value === kind)?.label || 'leave.kindIzin'
}

// Chips use the theme's own palette (chip-emerald / chip-lime / chip-dark) so the
// pills match the ones already on the shift, device and report tables instead of
// introducing a second set of status colours.
function kindChip(kind) {
    if (kind === 'cuti') return 'chip-emerald'
    if (kind === 'sakit') return 'chip-lime'
    return 'chip-dark'
}

function statusChip(status) {
    if (status === 'approved') return 'chip-emerald'
    if (status === 'rejected') return 'border-red-200 bg-red-50 text-red-700'
    return 'border-amber-200 bg-amber-50 text-amber-800'
}

// The mobile card's colour bar, mirroring the status chip it sits beside.
function statusBar(status) {
    if (status === 'approved') return 'bg-emerald-500'
    if (status === 'rejected') return 'bg-red-400'
    return 'bg-amber-400'
}

function statusLabel(status) {
    if (status === 'approved') return 'leave.stApproved'
    if (status === 'rejected') return 'leave.stRejected'
    return 'leave.stPending'
}

async function submit() {
    if (!chosenId.value) return notify.warning(t('leave.needsEmployee'), '')
    if (!/^\d{4}-\d{2}-\d{2}$/.test(newDate.value)) return notify.warning(t('leave.needsDate'), '')
    const resp = await leave.saveLeave({
        leave_employee_id: chosenId.value,
        leave_date: newDate.value,
        leave_kind: newKind.value,
        leave_reason: newReason.value.trim(),
    })
    if (resp.code === 200) {
        notify.success(t('leave.saved'), `${chosen.value ? `${chosen.value.first_name} ${chosen.value.last_name}`.trim() : ''} · ${newDate.value}`)
        newDate.value = ''
        newReason.value = ''
    } else {
        notify.error(t('leave.saveFailed'), Object.values(resp.data || {}).join(' '))
    }
}

async function decideRow(row, approve) {
    if (!approve && !confirm(t('leave.confirmReject', { name: displayName(row), date: String(row.leave_date).slice(0, 10) }))) return
    const resp = await leave.decide(row.leave_id, approve)
    if (resp.code === 200) {
        notify.success(approve ? t('leave.decidedApproved') : t('leave.decidedRejected'), displayName(row))
    } else {
        notify.error(t('leave.decideFailed'), displayName(row))
    }
}

async function cancelRow(row) {
    if (!confirm(t('leave.confirmCancel', { name: displayName(row), date: String(row.leave_date).slice(0, 10) }))) return
    const resp = await leave.cancelLeave(row.leave_id)
    if (resp.code === 200) notify.success(t('leave.cancelled'), displayName(row))
    else notify.error(t('leave.cancelFailed'), displayName(row))
}

onMounted(() => {
    leave.fetchLeaves()
    leave.fetchEmployees()
})
</script>

<template>
    <div class="space-y-4">
        <div
            class="hud-frame glass rounded-2xl border border-emerald-100 px-5 py-4 flex flex-col lg:flex-row lg:items-center gap-4 tech-grid-soft">
            <span class="hud-corner hud-corner-tl hidden sm:block"></span>
            <span class="hud-corner hud-corner-br hidden sm:block"></span>

            <div class="flex items-start gap-3">
                <div
                    class="hidden sm:flex w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-600 to-emerald-700 text-white items-center justify-center shadow-sm">
                    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
                        stroke-width="1.8">
                        <rect x="3" y="4" width="18" height="18" rx="2" />
                        <path d="M16 2v4M8 2v4M3 10h18" />
                        <path d="M9 16h6" />
                    </svg>
                </div>
                <div>
                    <h1 class="display text-lg font-extrabold text-green-900 tracking-tight">{{ $t('leave.title') }}
                    </h1>
                </div>
            </div>
            <div class="lg:ml-auto flex items-center gap-2">
                <input v-model="empQuery" type="search" :placeholder="$t('leave.filterEmployee')"
                    class="h-9 rounded-lg border-2 border-emerald-100 bg-white px-3 mono text-xs text-green-900 focus:outline-none focus:border-emerald-400">
                <select v-model="statusFilter"
                    class="h-9 rounded-lg border-2 border-emerald-100 bg-white px-2 mono text-xs text-green-900 focus:outline-none focus:border-emerald-400"
                    :aria-label="$t('leave.filterStatus')">
                    <option value="">{{ $t('leave.allStatuses') }}</option>
                    <option value="pending">{{ $t('leave.stPending') }}</option>
                    <option value="approved">{{ $t('leave.stApproved') }}</option>
                    <option value="rejected">{{ $t('leave.stRejected') }}</option>
                </select>
            </div>
        </div>

        <!-- Info text + Table: separate card -->
        <div class="glass rounded-2xl border border-emerald-100">
            <!-- Section label + pending count sit with the table, not in the HUD
                 header: that card is the filter bar, and a second heading there
                 competed with the page title for the same glance. -->
            <div
                class="flex items-center gap-2 px-4 py-3 border-b border-emerald-100 bg-gradient-to-r from-emerald-50/70 to-white">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                <span class="mono text-[10px] font-bold tracking-[0.14em] text-emerald-700 uppercase">{{
                    $t('leave.listTitle') }}</span>
                <span class="chip border-amber-300 bg-amber-50 text-amber-800 ml-auto">{{visible.filter(r =>
                    r.leave_status === 'pending').length }} {{ $t('leave.stPending') }}</span>
            </div>
            <p class="px-4 py-3 mono text-[11px] text-emerald-800 border-b border-emerald-100">
                {{ $t('leave.howItWorks') }}
            </p>

            <!-- List: same layout as the shift table -->
            <div class="hidden sm:block overflow-auto max-h-[360px]">
                <table class="w-full">
                    <thead class="sticky top-0 z-10">
                        <tr class="bg-gradient-to-r from-emerald-600 to-green-600 text-white">
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                                {{ $t('leave.colDate') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                                {{ $t('leave.colName') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                                {{ $t('leave.colKind') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15 hidden lg:table-cell">
                                {{ $t('leave.colReason') }}</th>
                            <th
                                class="py-2.5 px-3 text-left mono text-[10px] font-bold tracking-[0.14em] uppercase border-r border-white/15">
                                {{ $t('leave.colStatus') }}</th>
                            <th class="py-2.5 px-3 text-center mono text-[10px] font-bold tracking-[0.14em] uppercase">
                                {{ $t('common.actions') }}</th>
                        </tr>
                    </thead>
                    <tbody v-if="visible.length > 0" class="divide-y divide-emerald-50">
                        <tr v-for="row in visible" :key="row.leave_id"
                            class="group hover:bg-emerald-50/50 transition text-xs">
                            <td class="px-3 py-2.5 mono text-green-900 whitespace-nowrap">{{
                                String(row.leave_date).slice(0, 10) }}</td>
                            <td class="px-3 py-2.5">
                                <p class="font-bold text-green-900 leading-none">{{ displayName(row) }}</p>
                                <p v-if="row.requestedBy || row.request_by"
                                    class="mono text-[10px] text-emerald-700/50">{{ $t('leave.colRequestedBy') }}: {{
                                        row.requestedBy || row.request_by }}</p>
                            </td>
                            <td class="px-3 py-2.5">
                                <span class="chip" :class="kindChip(row.leave_kind)">{{ $t(kindLabel(row.leave_kind))
                                    }}</span>
                            </td>
                            <td class="px-3 py-2.5 text-emerald-900 max-w-[240px] truncate hidden lg:table-cell">{{
                                row.leave_reason }}</td>
                            <td class="px-3 py-2.5">
                                <span class="chip" :class="statusChip(row.leave_status)">{{
                                    $t(statusLabel(row.leave_status)) }}</span>
                            </td>
                            <td class="px-3 py-2.5 text-center">
                                <div class="inline-flex items-center gap-1">
                                    <template v-if="row.leave_status === 'pending'">
                                        <button type="button"
                                            class="px-2.5 py-1 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold tracking-wide disabled:opacity-40 transition"
                                            :disabled="leave.saving" @click="decideRow(row, true)">
                                            {{ $t('leave.approveBtn') }}
                                        </button>
                                        <button type="button"
                                            class="px-2.5 py-1 rounded-full bg-white border border-red-200 text-red-600 mono text-[10px] font-bold tracking-wide hover:bg-red-50 transition disabled:opacity-40"
                                            :disabled="leave.saving" @click="decideRow(row, false)">
                                            {{ $t('leave.rejectBtn') }}
                                        </button>
                                    </template>
                                    <button type="button"
                                        class="w-7 h-7 rounded-lg bg-white border border-red-100 text-red-500 hover:bg-red-500 hover:text-white hover:border-red-500 flex items-center justify-center transition"
                                        :title="$t('leave.cancelBtn')" @click="cancelRow(row)">
                                        <font-awesome :icon="faTrash" class="w-3 h-3" />
                                    </button>
                                </div>
                            </td>
                        </tr>
                    </tbody>
                    <tbody v-else>
                        <tr>
                            <td colspan="6" class="py-8 text-center">
                                <p class="mono text-xs text-emerald-700/50">{{ $t('leave.empty') }}</p>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <!-- Mobile cards -->
            <div class="sm:hidden p-2 space-y-2 max-h-[360px] overflow-auto">
                <div v-if="visible.length === 0"
                    class="py-8 text-center rounded-xl bg-emerald-50/40 border border-emerald-100 mono text-xs text-emerald-700/60">
                    {{ $t('leave.empty') }}</div>
                <div v-for="row in visible" :key="row.leave_id"
                    class="rounded-2xl border-2 border-emerald-100 bg-white p-3 flex items-center gap-3 active:border-emerald-300 active:bg-emerald-50/40">
                    <span class="w-2 h-10 rounded-full shrink-0" :class="statusBar(row.leave_status)"></span>
                    <div class="flex-1 min-w-0">
                        <p class="font-bold text-sm text-green-900 truncate">{{ displayName(row) }}</p>
                        <p class="mono text-xs text-emerald-700">{{ String(row.leave_date).slice(0, 10) }} <span
                                class="text-emerald-700/40">•</span> {{ $t(kindLabel(row.leave_kind)) }}</p>
                        <p v-if="row.leave_reason" class="mono text-[10px] text-slate-400 truncate">{{ row.leave_reason
                            }}</p>
                    </div>
                    <div class="flex flex-col gap-1 shrink-0">
                        <template v-if="row.leave_status === 'pending'">
                            <button type="button" @click="decideRow(row, true)"
                                class="px-3 py-1.5 rounded-full bg-emerald-600 text-white mono text-[10px] font-bold">{{
                                $t('leave.approveBtn') }}</button>
                            <button type="button" @click="decideRow(row, false)"
                                class="px-3 py-1.5 rounded-full bg-white border border-red-200 text-red-600 mono text-[10px]">{{
                                $t('leave.rejectBtn') }}</button>
                        </template>
                        <button type="button" @click="cancelRow(row)"
                            class="px-3 py-1.5 rounded-full bg-white border border-red-200 text-red-600 mono text-[10px]">{{
                            $t('leave.cancelBtn') }}</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Add new leave request -->
        <div class="glass rounded-2xl border border-emerald-100 px-4 py-3">
            <div class="mb-2">
                <span class="mono text-[11px] font-bold tracking-widest text-emerald-800">{{ $t('leave.addTitle')
                    }}</span>
            </div>
            <div class="grid gap-2 sm:grid-cols-[1fr_170px_210px_auto] sm:items-end">
                <div>
                    <label class="mono text-[10px] tracking-widest text-emerald-700/70">{{ $t('leave.who') }}</label>
                    <select v-model="chosenId"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-2 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                        <option :value="0">{{ $t('leave.whoPh') }}</option>
                        <option v-for="e in employeeOptions" :key="e.id" :value="e.id">
                            {{ `${e.first_name || ''} ${e.last_name || ''}`.trim() }}
                            <template v-if="e.departement"> - {{ e.departement }}</template>
                        </option>
                    </select>
                </div>
                <div>
                    <label class="mono text-[10px] tracking-widest text-emerald-700/70">{{ $t('leave.date') }}</label>
                    <input v-model="newDate" type="date"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-3 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                </div>
                <div>
                    <label class="mono text-[10px] tracking-widest text-emerald-700/70">{{ $t('leave.kind') }}</label>
                    <select v-model="newKind"
                        class="mt-1 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-2 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                        <option v-for="k in KINDS" :key="k.value" :value="k.value">{{ $t(k.label) }}</option>
                    </select>
                    <input v-model="newReason" type="text" :placeholder="$t('leave.reasonPh')"
                        class="mt-2 w-full h-10 rounded-lg border-2 border-emerald-100 bg-white px-3 text-sm text-green-900 focus:outline-none focus:border-emerald-400">
                </div>
                <button type="button"
                    class="btn-emerald h-10 px-4 rounded-lg text-sm font-bold inline-flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed transition"
                    :disabled="!canSubmit || leave.saving" @click="submit">
                    <font-awesome :icon="faCalendarPlus" class="w-3.5 h-3.5" />
                    <span>{{ $t('leave.submitBtn') }}</span>
                </button>
            </div>
        </div>
    </div>
</template>
