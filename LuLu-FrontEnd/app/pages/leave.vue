<script setup>
import { useLeaveStore } from '~/store/leave'
import { useAuthStore } from '~/store/auth'
import { useNotifyStore } from '~/store/notify'
import { faCalendarPlus, faTrash, faInbox, faMagnifyingGlass } from '@fortawesome/free-solid-svg-icons'
import { SMData } from '~/components/SimpleModal.vue'

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
// Form-only selection. Filtering the table must not follow this ref.
const whoId = ref(0)
const newDate = ref('')
const newKind = ref('izin')
const newReason = ref('')

const canSubmit = computed(() => whoId.value > 0 && /^\d{4}-\d{2}-\d{2}$/.test(newDate.value))


const chosen = computed(() => leave.employees.find((e) => e.id === whoId.value))

// Typeahead for the employee filter. Typing still searches the server by name;
// picking a row pins the filter to that employee id instead.
const filterSugOpen = ref(false)
const filterSugList = computed(() => {
  const q = empQuery.value.trim().toLowerCase()
  const people = leave.employees || []
  const matched = !q
    ? people
    : people.filter((e) => {
      const name = `${e.first_name || ''} ${e.last_name || ''}`.toLowerCase()
      return name.includes(q) || String(e.department || '').toLowerCase().includes(q)
    })
  return matched.slice(0, 8)
})

function onFilterFocus() {
  if (!leave.employees.length) leave.fetchEmployees()
  filterSugOpen.value = true
}

function onFilterInput() {
  // Typing again releases the pinned employee, back to free-text search.
  chosenId.value = 0
  filterSugOpen.value = true
}

// Same typeahead for the "who" field inside the request form. A native select
// becomes unusable once the roster passes a hundred people.
const whoQuery = ref('')
const whoOpen = ref(false)
const whoList = computed(() => {
  const q = whoQuery.value.trim().toLowerCase()
  const people = leave.employees || []
  const matched = !q
    ? people
    : people.filter((e) => {
      const name = `${e.first_name || ''} ${e.last_name || ''}`.toLowerCase()
      return name.includes(q) || String(e.department || '').toLowerCase().includes(q)
    })
  return matched.slice(0, 10)
})

function onWhoFocus() {
  if (!leave.employees.length) leave.fetchEmployees()
  whoOpen.value = true
}

function onWhoInput() {
  // Typing drops the previous pick so the form cannot submit a stale id.
  whoId.value = 0
  whoOpen.value = true
}

function pickWho(e) {
  whoId.value = e.id
  whoQuery.value = `${e.first_name || ''} ${e.last_name || ''}`.trim()
  whoOpen.value = false
}

function pickFilterEmployee(e) {
  chosenId.value = e.id
  empQuery.value = `${e.first_name || ''} ${e.last_name || ''}`.trim()
  filterSugOpen.value = false
}

// The server filters and pages; `visible` is just the current page.
const visible = computed(() => leave.items)
const page = ref(0)
const PAGE_SIZE = 50

async function loadLeaves() {
  await leave.fetchLeaves({
    employeeId: chosenId.value || undefined,
    q: chosenId.value ? undefined : empQuery.value.trim() || undefined,
    status: statusFilter.value || undefined,
    limit: PAGE_SIZE,
    offset: page.value * PAGE_SIZE,
  })
  // Deleting the last request of the last page leaves an empty page.
  if (!leave.items.length && page.value > 0 && leave.meta.total > 0) {
    page.value = Math.max(0, Math.ceil(leave.meta.total / PAGE_SIZE) - 1)
  }
}

let empTimer = null
watch(empQuery, () => {
  clearTimeout(empTimer)
  empTimer = setTimeout(() => { page.value = 0; loadLeaves() }, 300)
})
watch([chosenId, statusFilter], () => { page.value = 0; loadLeaves() })
function goPage(next) {
  if (next === page.value) return;
  page.value = next;
  loadLeaves();
}

function displayName(r) {
  return `${r.employee_fname || ''} ${r.employee_lname || ''}`.trim() || `#${r.leave_employee_id}`
}

// Same dd/mm/yyyy the payroll page renders; leave used to print raw ISO dates.
function displayDate(d) {
  if (!d) return '-'
  const s = String(d).slice(0, 10)
  if (/^\d{4}-\d{2}-\d{2}$/.test(s)) return new Date(s + 'T00:00:00Z').toLocaleDateString('id-ID')
  return s
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
  if (!whoId.value) return notify.warning(t('leave.needsEmployee'), '')
  if (!/^\d{4}-\d{2}-\d{2}$/.test(newDate.value)) return notify.warning(t('leave.needsDate'), '')
  const resp = await leave.saveLeave({
    leave_employee_id: whoId.value,
    leave_date: newDate.value,
    leave_kind: newKind.value,
    leave_reason: newReason.value.trim(),
  })
  if (resp.code === 200) {
    notify.success(t('leave.saved'), `${chosen.value ? `${chosen.value.first_name} ${chosen.value.last_name}`.trim() : ''} · ${newDate.value}`)
    newDate.value = ''
    newReason.value = ''
  } else {
    // A 409 (decided request) carries no field errors: surface its message,
    // translated when the backend sent an i18n key.
    const fieldErrors = Array.isArray(resp.data) ? '' : Object.values(resp.data || {}).join(' ')
    const reason = resp.error ? t(resp.error) : (fieldErrors || resp.message || '')
    notify.error(t('leave.saveFailed'), reason)
  }
}

// Rejecting and withdrawing are irreversible, so both go through the shared
// confirmation modal instead of window.confirm (which blocks the tab and cannot
// be styled or translated consistently).
const confirmModal = new SMData()
const pendingAction = ref(null)

function askConfirm(row, action) {
  pendingAction.value = { action, row }
  const args = { name: displayName(row), date: displayDate(row.leave_date) }
  if (action === 'reject') confirmModal.setText(t('leave.rejectTitle'), t('leave.confirmReject', args))
  else confirmModal.setText(t('leave.cancelTitle'), t('leave.confirmCancel', args))
  confirmModal.showOKCancel()
}

async function decideRow(row, approve) {
  if (!approve) return askConfirm(row, 'reject')
  const resp = await leave.decide(row.leave_id, true)
  if (resp.code === 200) notify.success(t('leave.decidedApproved'), displayName(row))
  else notify.error(t('leave.decideFailed'), displayName(row))
}

function cancelRow(row) {
  askConfirm(row, 'cancel')
}

async function runPendingAction() {
  const pending = pendingAction.value
  confirmModal.clear()
  pendingAction.value = null
  if (!pending) return

  if (pending.action === 'reject') {
    const resp = await leave.decide(pending.row.leave_id, false)
    if (resp.code === 200) notify.success(t('leave.decidedRejected'), displayName(pending.row))
    else notify.error(t('leave.decideFailed'), displayName(pending.row))
    return
  }
  const resp = await leave.cancelLeave(pending.row.leave_id)
  if (resp.code === 200) notify.success(t('leave.cancelled'), displayName(pending.row))
  else notify.error(t('leave.cancelFailed'), displayName(pending.row))
}

function abortPendingAction() {
  pendingAction.value = null
  confirmModal.clear()
}

onMounted(() => {
  loadLeaves()
  leave.fetchEmployees()
})
</script>

<template>
  <div class="flex flex-col gap-4">

    <!-- page head: plain title row, never a card -->
    <div class="flex flex-col sm:flex-row sm:items-center gap-3">
      <div class="min-w-0">
        <h1 class="display text-[20px] text-text">{{ $t('leave.title') }}</h1>
        <div class="mt-1.5 flex flex-wrap items-center gap-1.5">
          <span class="chip chip-quiet">{{ leave.meta.total }} {{ $t('leave.recordsUnit') }}</span>
          <span class="chip chip-warn">{{ leave.meta.pending || 0 }} {{ $t('leave.stPending') }}</span>
          <span class="chip chip-ok">{{ leave.meta.approved || 0 }} {{ $t('leave.stApproved') }}</span>
        </div>
      </div>
    </div>

    <!-- controls -->
    <div class="panel p-3 sm:p-4" style="overflow: visible">
      <div class="flex flex-col sm:flex-row gap-2 sm:items-center">
        <div class="relative w-full flex-1 min-w-[240px] max-w-[440px]">
          <font-awesome :icon="faMagnifyingGlass"
            class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
          <input v-model="empQuery" type="search" :placeholder="$t('leave.filterEmployee')" class="field has-icon-left"
            :aria-label="$t('leave.filterEmployee')" autocomplete="off" role="combobox" :aria-expanded="filterSugOpen"
            @focus="onFilterFocus" @input="onFilterInput" @blur="filterSugOpen = false"
            @keydown.esc.prevent="filterSugOpen = false">
          <ul v-if="filterSugOpen && filterSugList.length"
            class="absolute z-30 mt-1 w-full max-h-64 overflow-auto rounded-xl border border-border bg-surface shadow-lg p-1">
            <li v-for="e in filterSugList" :key="`fs-${e.id}`">
              <button type="button" class="w-full text-left px-2.5 py-2 rounded-lg hover:bg-surface-raised"
                @mousedown.prevent="pickFilterEmployee(e)">
                <span class="block text-[13px] font-semibold text-text truncate">
                  {{ [e.first_name || e.name, e.last_name].filter(Boolean).join(' ') }}
                </span>
                <span class="block mono text-[10px] text-text-muted truncate mt-0.5">{{ e.department || '-' }}</span>
              </button>
            </li>
          </ul>
        </div>
        <select v-model="statusFilter" class="field sm:w-56" :aria-label="$t('leave.filterStatus')">
          <option value="">{{ $t('leave.allStatuses') }}</option>
          <option value="pending">{{ $t('leave.stPending') }}</option>
          <option value="approved">{{ $t('leave.stApproved') }}</option>
          <option value="rejected">{{ $t('leave.stRejected') }}</option>
        </select>
        <span class="mono text-[10.5px] text-text-muted sm:ml-auto">
          {{ visible.length }} / {{ leave.meta.total }} {{ $t('leave.recordsUnit') }}
        </span>
      </div>
    </div>

    <!-- requests -->
    <div class="panel overflow-hidden">
      <div class="panel-head">
        <span class="inline-flex items-center gap-2 min-w-0">
          <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
          <span class="truncate">{{ $t('leave.listTitle') }}</span>
        </span>
        <span class="chip chip-warn shrink-0">{{ leave.meta.pending || 0 }} {{ $t('leave.stPending') }}</span>
      </div>

      <!-- desktop table -->
      <div class="hidden md:block table-shell max-h-[480px]">
        <table class="w-full min-w-[860px]">
          <thead class="sticky top-0 z-10">
            <tr>
              <th class="th">{{ $t('leave.colDate') }}</th>
              <th class="th">{{ $t('leave.colName') }}</th>
              <th class="th">{{ $t('leave.colKind') }}</th>
              <th class="th hidden lg:table-cell">{{ $t('leave.colReason') }}</th>
              <th class="th">{{ $t('leave.colStatus') }}</th>
              <th class="th !text-right">{{ $t('common.actions') }}</th>
            </tr>
          </thead>

          <tbody v-if="visible.length > 0">
            <tr v-for="row in visible" :key="row.leave_id" class="even:bg-surface-raised/25">
              <td class="td whitespace-nowrap mono text-[11.5px] font-bold">{{ displayDate(row.leave_date) }}</td>

              <td class="td">
                <p class="text-[13px] font-semibold text-text truncate">{{ displayName(row) }}</p>
                <p v-if="row.requestedBy || row.request_by" class="mono text-[10px] text-text-muted truncate">
                  {{ $t('leave.colRequestedBy') }}: {{ row.requestedBy || row.request_by }}
                </p>
              </td>

              <td class="td">
                <span class="chip chip-quiet">{{ $t(kindLabel(row.leave_kind)) }}</span>
              </td>

              <td class="td hidden lg:table-cell max-w-[260px]">
                <p class="text-[12.5px] text-text-muted truncate" :title="row.leave_reason">{{ row.leave_reason }}</p>
              </td>

              <td class="td">
                <span class="chip"
                  :class="row.leave_status === 'approved' ? 'chip-ok' : row.leave_status === 'rejected' ? 'chip-danger' : 'chip-warn'">{{
                    $t(statusLabel(row.leave_status)) }}</span>
              </td>

              <td class="td">
                <div class="flex items-center justify-end gap-1.5">
                  <template v-if="row.leave_status === 'pending'">
                    <button type="button" class="btn btn-primary btn-sm !min-h-[44px]" :disabled="leave.saving"
                      @click="decideRow(row, true)">
                      {{ $t('leave.approveBtn') }}
                    </button>
                    <button type="button" class="btn btn-danger btn-sm !min-h-[44px]" :disabled="leave.saving"
                      @click="decideRow(row, false)">
                      {{ $t('leave.rejectBtn') }}
                    </button>
                    <button type="button" class="btn btn-quiet btn-sm !min-h-[44px] !w-11 !px-0"
                      :title="$t('leave.cancelBtn')" :aria-label="$t('leave.cancelBtn')" @click="cancelRow(row)">
                      <font-awesome :icon="faTrash" class="w-3.5 h-3.5" />
                    </button>
                  </template>
                </div>
              </td>
            </tr>
          </tbody>

          <tbody v-else>
            <tr>
              <td colspan="6" class="p-0">
                <div class="state">
                  <div class="state-icon">
                    <span class="hud-corner hud-corner-tl"></span>
                    <span class="hud-corner hud-corner-br"></span>
                    <font-awesome :icon="faInbox" class="w-5 h-5" />
                  </div>
                  <p class="state-title">{{ $t('leave.empty') }}</p>
                  <p class="state-hint">{{ $t('leave.howItWorksShort') }}</p>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- mobile cards -->
      <div class="md:hidden p-3 space-y-2.5">
        <div v-if="visible.length === 0" class="state">
          <div class="state-icon">
            <span class="hud-corner hud-corner-tl"></span>
            <span class="hud-corner hud-corner-br"></span>
            <font-awesome :icon="faInbox" class="w-5 h-5" />
          </div>
          <p class="state-title">{{ $t('leave.empty') }}</p>
          <p class="state-hint">{{ $t('leave.howItWorksShort') }}</p>
        </div>

        <div v-for="row in visible" :key="row.leave_id" class="rounded-xl border border-border bg-surface p-3">
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <p class="text-[13px] font-semibold text-text truncate">{{ displayName(row) }}</p>
              <p class="mono text-[10.5px] text-text-muted">
                {{ displayDate(row.leave_date) }} <span class="opacity-60">·</span> {{ $t(kindLabel(row.leave_kind)) }}
              </p>
            </div>
            <span class="chip shrink-0"
              :class="row.leave_status === 'approved' ? 'chip-ok' : row.leave_status === 'rejected' ? 'chip-danger' : 'chip-warn'">{{
                $t(statusLabel(row.leave_status)) }}</span>
          </div>

          <p v-if="row.leave_reason" class="mt-2 text-[11.5px] text-text-muted">{{ row.leave_reason }}</p>
          <p v-if="row.requestedBy || row.request_by" class="mt-1 mono text-[10px] text-text-muted">
            {{ $t('leave.colRequestedBy') }}: {{ row.requestedBy || row.request_by }}
          </p>

          <div v-if="row.leave_status === 'pending'" class="mt-2.5 flex flex-wrap gap-1.5">
            <button type="button" @click="decideRow(row, true)" class="btn btn-primary flex-1">{{ $t('leave.approveBtn')
              }}</button>
            <button type="button" @click="decideRow(row, false)" class="btn btn-danger flex-1">{{ $t('leave.rejectBtn')
              }}</button>
            <button type="button" @click="cancelRow(row)" class="btn btn-secondary flex-1">{{ $t('leave.cancelBtn')
              }}</button>
          </div>
        </div>
      </div>

      <TablePager :page="page" :page-size="PAGE_SIZE" :total="leave.meta.total" @update:page="goPage" />

      <p class="px-4 py-3 border-t border-border mono text-[10.5px] text-text-muted">
        {{ $t('leave.howItWorks') }}
      </p>
    </div>

    <!-- new request -->
    <div class="panel" style="overflow: visible">
      <div class="panel-head">
        <span class="inline-flex items-center gap-2 min-w-0">
          <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
          <span class="truncate">{{ $t('leave.addTitle') }}</span>
        </span>
        <span class="mono text-[10.5px] normal-case tracking-normal text-text-muted shrink-0">{{
          $t('leave.howItWorksShort')
          }}</span>
      </div>

      <div class="panel-body">
        <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-[1fr_180px_220px_auto] lg:items-end">
          <div class="relative">
            <label class="label" for="leave-who">{{ $t('leave.who') }}</label>
            <input id="leave-who" v-model="whoQuery" type="text" autocomplete="off" class="field"
              :placeholder="$t('leave.whoPh')" role="combobox" :aria-expanded="whoOpen" @focus="onWhoFocus"
              @input="onWhoInput" @blur="whoOpen = false" @keydown.esc.prevent="whoOpen = false" />
            <ul v-if="whoOpen && whoList.length"
              class="absolute z-30 mt-1 w-full max-h-64 overflow-auto rounded-xl border border-border bg-surface shadow-lg p-1">
              <li v-for="e in whoList" :key="`who-${e.id}`">
                <button type="button" class="w-full text-left px-2.5 py-2 rounded-lg hover:bg-surface-raised"
                  @mousedown.prevent="pickWho(e)">
                  <span class="block text-[13px] font-semibold text-text truncate">
                    {{ [e.first_name || e.name, e.last_name].filter(Boolean).join(' ') }}
                  </span>
                  <span class="block mono text-[10px] text-text-muted truncate mt-0.5">{{ e.department || '-' }}</span>
                </button>
              </li>
            </ul>
          </div>

          <div>
            <label class="label" for="leave-date">{{ $t('leave.date') }}</label>
            <input id="leave-date" v-model="newDate" type="date" class="field">
          </div>

          <div>
            <label class="label" for="leave-kind">{{ $t('leave.kind') }}</label>
            <select id="leave-kind" v-model="newKind" class="field">
              <option v-for="k in KINDS" :key="k.value" :value="k.value">{{ $t(k.label) }}</option>
            </select>
            <input v-model="newReason" type="text" :placeholder="$t('leave.reasonPh')" class="field mt-2"
              :aria-label="$t('leave.reason')">
          </div>

          <button type="button" class="btn btn-primary w-full lg:w-auto" :disabled="!canSubmit || leave.saving"
            @click="submit">
            <font-awesome :icon="faCalendarPlus" class="w-3.5 h-3.5" />
            <span>{{ $t('leave.submitBtn') }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
  <SimpleModal :options="confirmModal.get()" @ok="runPendingAction" @cancel="abortPendingAction"
    v-show="confirmModal.isShow()" />
</template>
