<script setup>
import { useEmployee } from '~/store/employee';
import { useAuthStore } from '~/store/auth';
import { useSchedule } from '~/store/schedule';
import { faInbox, faFileCsv, faRotate, faCircleExclamation } from '@fortawesome/free-solid-svg-icons';

const
    auth = useAuthStore(),
    config = useRuntimeConfig(),
    list = useEmployee(),
    sched = useSchedule(),
    emit = defineEmits(['delete', 'edit', 'import', 'load-error']),
    props = defineProps({
        search: { type: String, default: '' },
        deptFilter: { type: String, default: '' },
        statusFilter: { type: String, default: '' },
    });

// List rows only expose writes the signed-in user is actually allowed to make.
// Declared after `auth` exists: a computed callback referencing `auth` above its
// declaration is fine at definition time but throws on first access.
const canEdit = computed(() => auth.hasActionAccess('employee', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('employee', 'can_delete'))

const scheduleMap = computed(() => {
    const m = {}
    for (const s of (sched.contents || [])) {
        if (String(s.parent) === String(s.id)) m[String(s.parent)] = s
    }
    return m
})
function scheduleNameFor(id) {
    if (id == null || id === '') return '-'
    const s = scheduleMap.value[String(id)]
    return s ? s.name : `#${id}`
}
function scheduleBadgeFor(id) {
    const s = scheduleMap.value[String(id)]
    if (!s) return ''
    if (s.type === 1) return $t('report.periodDaily')
    if (s.type === 7) return $t('report.periodWeekly')
    if (s.type === 14) return $t('report.period2M')
    if (s.type === 21) return $t('report.period3M')
    if (s.type === 28) return $t('report.periodMonthly')
    return `T${s.type}`
}
function initials(a, b) {
    return `${(a || '?')[0] || ''}${(b || '')[0] || ''}`.toUpperCase().slice(0, 2)
}
function avatarBg(item) {
    const s = `${item.first_name || ''}${item.last_name || ''}`
    let h = 0; for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) % 360
    return `hsl(${140 + (h % 40) - 20} 70% 38%)`
}

// The API pages and filters; this component only holds the current page.
const filtered = computed(() => list.contents || [])
const total = computed(() => list.meta.all_total || 0)
const filteredCount = computed(() => list.meta.total || filtered.value.length)
const page = ref(0)
const PAGE_SIZE = 50

function callbackDelete(id) { emit('delete', id) }
function callbackEdit(id) { emit('edit', id) }

const loadError = ref(false)

async function initEmployee() {
    loadError.value = false
    emit('load-error', false)
    try {
        const params = new URLSearchParams({ limit: String(PAGE_SIZE), offset: String(page.value * PAGE_SIZE) })
        const q = (props.search || '').trim()
        if (q) params.set('q', q)
        if (props.deptFilter) params.set('department', props.deptFilter)
        if (props.statusFilter !== '' && props.statusFilter !== null && props.statusFilter !== undefined) {
            params.set('status', String(props.statusFilter))
        }
        const r = await apiFetch(`${config.public.apiBase}/employee/list?${params}`, { method: 'GET', headers: auth.confHeaders() })
        if (r.code === 200) {
            // Paged shape is a one-item envelope; a plain array is the legacy list.
            const payload = Array.isArray(r.data) && Array.isArray(r.data[0]?.items)
                ? r.data[0]
                : { items: Array.isArray(r.data) ? r.data : [] }
            list.set(payload.items || [])
            list.setMeta(payload)
            // Deleting the last row of the last page leaves an empty page.
            if (!(payload.items || []).length && page.value > 0 && (payload.total || 0) > 0) {
                page.value = Math.max(0, Math.ceil(payload.total / PAGE_SIZE) - 1)
            }
        }
        else { loadError.value = true; emit('load-error', true) }
    } catch (e) { loadError.value = true; emit('load-error', true) }
    try {
        const r2 = await apiFetch(`${config.public.apiBase}/schedule/list`, { method: 'GET', headers: auth.confHeaders() })
        if (r2.code === 200) sched.set(r2.data)
    } catch (e) { }
}
function reload() { initEmployee() }

// Filter changes restart from the first page; only typing waits for a pause.
let searchTimer = null
watch(() => props.search, () => {
    clearTimeout(searchTimer)
    searchTimer = setTimeout(() => { page.value = 0; initEmployee() }, 300)
})
watch([() => props.deptFilter, () => props.statusFilter], () => {
    page.value = 0
    initEmployee()
})
function goPage(next) {
    if (next === page.value) return
    page.value = next
    initEmployee()
}

// Import reads the file in the browser and POSTs it as a plain JSON string. A
// multipart upload would need an extra dependency and a second parser on the
// server for no gain: the CSV is small, and text is what csv::Reader wants.
async function onImportFile(e) {
    const file = e.target.files?.[0]
    e.target.value = '' // clear so re-picking the same file still fires change
    if (!file) return
    const csv = await file.text()
    try {
        const r = await apiFetch(`${config.public.apiBase}/employee/import`, { method: 'POST', headers: auth.confHeaders(), body: { csv } })
        emit('import', r)
        reload()
    } catch (err) {
        emit('import', { code: 500, message: err?.data?.message || 'Import failed', data: [] })
    }
}
defineExpose({ reload, initEmployee })

onMounted(() => { initEmployee() })
</script>

<template>
    <div class="flex flex-col overflow-hidden">

        <!-- toolbar -->
        <div class="panel-head">
            <span class="inline-flex items-center gap-2 min-w-0">
                <span class="w-1.5 h-1.5 rounded-full bg-primary shrink-0"></span>
                <span class="truncate">{{ $t('employee.statEmployees') }}</span>
            </span>
            <!--
                One right-hand group, not justify-between: with four children
                justify-between spreads them across the full width, so the
                buttons drifted to opposite edges. Import sits left of Reload
                because both act on the list itself, while Add opens the form.
            -->
            <div class="ml-auto flex flex-wrap items-center justify-end gap-1.5 shrink-0 normal-case tracking-normal">
                <span v-if="!loadError" class="mono text-[10px] text-text-muted hidden sm:inline">{{
                    $t('employee.countShownTotal', { shown: filteredCount, total: total }) }}</span>
                <label class="btn btn-secondary btn-sm cursor-pointer" :title="$t('employee.importTitle')">
                    <font-awesome :icon="faFileCsv" class="w-3.5 h-3.5" />
                    <span class="hidden sm:inline">{{ $t('employee.importTitle') }}</span>
                    <input type="file" accept=".csv,text/csv" class="hidden" @change="onImportFile">
                </label>
                <button @click="reload()" :title="$t('device.reloadShort')" class="btn btn-primary btn-sm">
                    <font-awesome :icon="faRotate" class="w-3.5 h-3.5" />
                    <span class="hidden sm:inline">{{ $t('device.reloadShort') }}</span>
                </button>
            </div>
        </div>

        <!-- load failure: never a red browser alert, and never a false zero in the header -->
        <div v-if="loadError" class="p-3">
            <div class="state">
                <div class="state-icon !border-danger/30 !text-danger">
                    <span class="hud-corner hud-corner-tl"></span>
                    <span class="hud-corner hud-corner-br"></span>
                    <font-awesome :icon="faCircleExclamation" class="w-5 h-5" />
                </div>
                <p class="state-title !text-danger">{{ $t('common.loadFailed') }}</p>
                <p class="state-hint">{{ $t('employee.loadFailedHint') }}</p>
                <button @click="reload()" class="btn btn-secondary btn-sm mt-1">
                    <font-awesome :icon="faRotate" class="w-3 h-3" />
                    {{ $t('common.retry') }}
                </button>
            </div>
        </div>

        <!-- ===== desktop table ===== -->
        <div v-show="!loadError" class="table-shell max-h-[56vh] md:max-h-[58vh]">
            <table class="w-full min-w-[780px] border-collapse hidden md:table">
                <thead class="sticky top-0 z-10">
                    <tr>
                        <th class="th">{{ $t('employee.colEmployee') }}</th>
                        <th class="th">{{ $t('employee.department') }}</th>
                        <th class="th">{{ $t('employee.schedule') }}</th>
                        <th class="th !text-center">{{ $t('employee.status') }}</th>
                        <th class="th !text-right">{{ $t('common.actions') }}</th>
                    </tr>
                </thead>

                <tbody v-if="filteredCount > 0">
                    <tr v-for="data in filtered" :key="data.id" class="even:bg-surface-raised/25">
                        <td class="td">
                            <div class="flex items-center gap-2.5 min-w-0">
                                <div class="w-8 h-8 rounded-lg grid place-content-center text-white mono text-[11px] font-extrabold shrink-0"
                                    :style="{ background: avatarBg(data) }">{{ initials(data.first_name, data.last_name)
                                    }}</div>
                                <div class="min-w-0">
                                    <p class="text-[13px] font-semibold text-text truncate">{{ data.first_name }} {{
                                        data.last_name || '' }}</p>
                                    <p class="mono text-[10px] text-text-muted truncate">
                                        #{{ data.id }} <span class="opacity-50">·</span> {{ data.address ||
                                        $t('employee.noAddress') }}
                                    </p>
                                </div>
                            </div>
                        </td>

                        <td class="td">
                            <span class="chip chip-quiet">{{ data.department || '-' }}</span>
                        </td>

                        <td class="td">
                            <span v-if="data.schedule_id" class="chip chip-emerald">
                                <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
                                {{ scheduleNameFor(data.schedule_id) }}
                                <span class="opacity-60">{{ scheduleBadgeFor(data.schedule_id) }}</span>
                            </span>
                            <span v-else class="chip chip-quiet">{{ $t('employee.noSchedule') }}</span>
                        </td>

                        <td class="td !text-center">
                            <span class="chip" :class="String(data.status) === '0' ? 'chip-ok' : 'chip-warn'">{{
                                String(data.status) === '0' ? $t('common.active') : $t('employee.statusInactive')
                                }}</span>
                        </td>

                        <td class="td">
                            <div class="flex items-center justify-end gap-1.5 whitespace-nowrap">
                                <button v-if="canEdit" @click="callbackEdit(data.id)"
                                    class="btn btn-secondary btn-sm">{{ $t('common.edit') }}</button>
                                <button v-if="canDelete" @click="callbackDelete(data.id)"
                                    class="btn btn-danger btn-sm">{{ $t('common.delete') }}</button>
                            </div>
                        </td>
                    </tr>
                </tbody>

                <tbody v-else>
                    <tr>
                        <td colspan="5" class="p-0">
                            <div class="state">
                                <div class="state-icon">
                                    <span class="hud-corner hud-corner-tl"></span>
                                    <span class="hud-corner hud-corner-br"></span>
                                    <font-awesome :icon="faInbox" class="w-5 h-5" />
                                </div>
                                <p class="state-title">{{ (props.search || props.deptFilter || props.statusFilter !== '') ?
                                    $t('employee.notFoundTitle') : $t('employee.noneYet') }}</p>
                                <p v-if="props.search || props.deptFilter || props.statusFilter !== ''" class="state-hint">{{
                                    $t('employee.tryKeyword') }}</p>
                            </div>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <!-- ===== mobile cards ===== -->
        <div class="md:hidden p-3 space-y-2.5 max-h-[56vh] overflow-auto" v-if="!loadError && filteredCount > 0">
            <div v-for="data in filtered" :key="`m-${data.id}`" class="rounded-xl border border-border bg-surface p-3">
                <div class="flex items-start justify-between gap-2">
                    <div class="flex items-center gap-2.5 min-w-0">
                        <div class="w-8 h-8 rounded-lg grid place-content-center text-white mono text-[11px] font-extrabold shrink-0"
                            :style="{ background: avatarBg(data) }">{{ initials(data.first_name, data.last_name) }}</div>
                        <div class="min-w-0">
                            <p class="text-[13px] font-semibold text-text truncate">{{ data.first_name }} {{
                                data.last_name||'' }}</p>
                            <p class="mono text-[10px] text-text-muted">#{{ data.id }}</p>
                        </div>
                    </div>
                    <span class="chip shrink-0" :class="String(data.status) === '0' ? 'chip-ok' : 'chip-warn'">{{
                        String(data.status) === '0' ? $t('common.active') : $t('employee.statusInactive') }}</span>
                </div>

                <div class="mt-2.5 flex flex-wrap gap-1.5">
                    <span class="chip chip-quiet">{{ data.department || '-' }}</span>
                    <span v-if="data.schedule_id" class="chip chip-emerald">
                        <span class="w-1.5 h-1.5 rounded-full bg-primary"></span>
                        {{ scheduleNameFor(data.schedule_id) }} <span class="opacity-60">{{
                            scheduleBadgeFor(data.schedule_id) }}</span>
                    </span>
                    <span v-else class="chip chip-quiet">{{ $t('employee.noSchedule') }}</span>
                </div>

                <p v-if="data.address" class="mt-2 text-[12px] text-text-muted line-clamp-2">{{ data.address }}</p>

                <div class="mt-2.5 pt-2.5 border-t border-border flex gap-1.5">
                    <button v-if="canEdit" @click="callbackEdit(data.id)" class="btn btn-secondary flex-1">{{
                        $t('common.edit') }}</button>
                    <button v-if="canDelete" @click="callbackDelete(data.id)" class="btn btn-danger flex-1">{{
                        $t('common.delete') }}</button>
                </div>
            </div>
        </div>

        <div v-else-if="!loadError" class="md:hidden">
            <div class="state">
                <div class="state-icon">
                    <span class="hud-corner hud-corner-tl"></span>
                    <span class="hud-corner hud-corner-br"></span>
                    <font-awesome :icon="faInbox" class="w-5 h-5" />
                </div>
                <p class="state-title">{{ (props.search || props.deptFilter || props.statusFilter !== '') ?
                    $t('employee.notFoundTitle') : $t('employee.noneYet') }}</p>
                <p v-if="props.search || props.deptFilter || props.statusFilter !== ''" class="state-hint">{{
                    $t('employee.tryKeyword') }}</p>
            </div>
        </div>

        <TablePager :page="page" :page-size="PAGE_SIZE" :total="filteredCount" @update:page="goPage" />

        <div v-if="filteredCount > 0"
            class="px-4 py-2.5 border-t border-border bg-surface flex items-center justify-between gap-2 mono text-[10px] text-text-muted">
            <span>{{ $t('employee.countShownTotal', { shown: filteredCount, total: total }) }}</span>
            <span class="hidden sm:inline">{{ $t('employee.footerHint') }}</span>
        </div>
    </div>
</template>
