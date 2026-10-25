<script setup>
import { useDeviceStore } from '~/store/devices';
import { useAuthStore } from '~/store/auth';
import { useChecker, useFormater } from '#imports';
import { useI18n } from 'vue-i18n';
import { faMagnifyingGlass, faPen, faTrash, faRotate, faServer, faDownload, faBan, faXmark } from '@fortawesome/free-solid-svg-icons';

const
  auth = useAuthStore(),
  config = useRuntimeConfig(),
  list = useDeviceStore(),
  check = useChecker(),
  format = useFormater(),
  { t } = useI18n(),
  { $notify: notify } = useNuxtApp(),
  emit = defineEmits(['delete', 'edit']),
  search = ref(''),
  statusFilter = ref('all'),
  loading = ref(false),
  pulling = ref({});

const canEdit = computed(() => auth.hasActionAccess('controller', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('controller', 'can_delete'))

const contents = computed(() => list.contents)
const filtered = computed(() => {
  let out = contents.value
  const q = search.value.trim().toLowerCase()
  if (q) out = out.filter(d =>
    (d.name || '').toLowerCase().includes(q) ||
    (d.sn || '').toLowerCase().includes(q) ||
    (d.location || '').toLowerCase().includes(q)
  )
  if (statusFilter.value !== 'all') {
    const map = { online: 1, offline: 2, disable: 3 }
    const code = map[statusFilter.value]
    if (code !== undefined) out = out.filter(d => Number(d.status) === code)
  }
  return out
})
const hasFilter = computed(() => search.value.trim() !== '' || statusFilter.value !== 'all')
function clearFilter() { search.value = ''; statusFilter.value = 'all' }

async function initListController() {
  loading.value = true
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/list`, { method: "GET", headers: auth.confHeaders() });
    if (response.code == 200) list.set(response.data);
  } catch (e) { console.error(e) } finally { loading.value = false }
}

function callbackDelete(id) { emit('delete', id) }
function callbackEdit(id) { emit('edit', id) }

function statusMeta(s) {
  const n = Number(s)
  // `tone` and `dot` are for a flat text+badge treatment: colour carries the
  // state, the label carries the meaning.
  switch (n) {
    case 1: return { label: $t('device.stOnline'), cls: 'bg-emerald-500 text-white border-emerald-500', tone: 'text-primary-deep', dot: 'bg-primary', ring: 'ring-emerald-200' }
    case 2: return { label: $t('device.stOffline'), cls: 'bg-amber-500 text-white border-amber-500', tone: 'text-warn', dot: 'bg-amber-500', ring: 'ring-amber-200' }
    case 3: return { label: $t('device.stDisable'), cls: 'bg-slate-400 text-white border-slate-400', tone: 'text-text-muted', dot: 'bg-slate-400', ring: 'ring-slate-200' }
    case 4: return { label: $t('device.stSync'), cls: 'bg-sky-500 text-white border-sky-500', tone: 'text-primary', dot: 'bg-primary', ring: 'ring-sky-200' }
    default: return { label: $t('device.stUnknown'), cls: 'bg-white text-emerald-700 border-emerald-200', tone: 'text-text-muted', dot: 'bg-slate-300', ring: '' }
  }
}
function handlerLabel(h) {
  if (h == null || h === '') return '-'
  const s = String(h).toLowerCase()
  if (s === '4' || s.includes('iclock') || s.includes('adms')) return 'iClock / ADMS'
  if (s === '5' || s.includes('zknet')) return 'ZKNET'
  return String(h)
}
function handlerTone(h) {
  const l = handlerLabel(h)
  if (l.includes('iClock')) return 'bg-emerald-50 border-emerald-200 text-emerald-700'
  if (l.includes('ZKNET')) return 'bg-amber-50 border-amber-200 text-amber-700'
  return 'bg-white border-emerald-200 text-emerald-800'
}
function tzLabel(tz) {
  const n = Number(tz)
  if (Number.isNaN(n)) return String(tz)
  if (n === 7) return $t('device.tzWib')
  if (n === 8) return $t('device.tzWita')
  if (n === 9) return $t('device.tzWit')
  const sign = n >= 0 ? '+' : ''
  return `GMT${sign}${n}`
}
function lastSyncText(v) {
  if (!v || v === 0 || v === '0') return '-'
  try { return format.stamp_to_naive(v) } catch { return String(v) }
}

async function toggleDevice(id) {
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/disable/${id}`, {
      method: "GET",
      headers: auth.confHeaders()
    });
    if (response.code === 200) {
      notify?.success(response.message || t('device.pullSuccess'));
      await initListController();
    } else {
      notify?.error(response.message || t('device.pullFailed'));
    }
  } catch (e) {
    console.error(e);
    notify?.error(t('device.pullFailed'));
  }
}

async function pullDevice(id) {
  pulling.value[id] = true;
  try {
    const response = await apiFetch(`${config.public.apiBase}/device/pull/${id}`, { method: "POST", headers: auth.confHeaders() });
    if (response.code === 200) {
      notify?.success(response.message || t('device.pullSuccess'));
      await initListController();
    } else {
      // Surface the server's reason (device unreachable, wrong handler,
      // unreadable payload) instead of a generic failure toast.
      notify?.error(response.message || t('device.pullFailed'));
    }
  } catch (e) {
    console.error(e);
    notify?.error(t('device.pullFailed'));
  } finally {
    pulling.value[id] = false;
  }
}

onMounted(() => initListController());
defineExpose({ reload: initListController })
</script>

<template>
  <div class="flex flex-col min-h-0">

    <!-- toolbar -->
    <div class="px-3 py-2.5 border-b border-border flex flex-col sm:flex-row gap-2 sm:items-center">
      <div class="relative w-full flex-1 min-w-[240px] max-w-[440px]">
        <font-awesome :icon="faMagnifyingGlass"
          class="w-3.5 h-3.5 absolute left-3 top-1/2 -translate-y-1/2 text-text-muted pointer-events-none" />
        <input v-model="search" type="text" :placeholder="$t('device.searchPh')"
          class="field has-icon-left has-icon-right !min-h-[40px]" :aria-label="$t('device.searchPh')" />
        <button v-if="search" @click="search = ''"
          class="absolute right-1 top-1/2 -translate-y-1/2 w-8 h-8 grid place-content-center rounded-lg text-text-muted hover:bg-surface-raised transition"
          :aria-label="$t('common.clearSearch')">
          <font-awesome :icon="faXmark" class="w-3 h-3" />
        </button>
      </div>

      <div class="flex items-center gap-2 shrink-0">
        <select v-model="statusFilter" class="field !min-h-[40px] !w-auto text-[12.5px]"
          :aria-label="$t('report.allStatuses')">
          <option value="all">{{ $t('report.allStatuses') }}</option>
          <option value="online">{{ $t('device.stOnline') }}</option>
          <option value="offline">{{ $t('device.stOffline') }}</option>
          <option value="disable">{{ $t('device.stDisable') }}</option>
        </select>

        <button v-if="hasFilter" @click="clearFilter()" class="btn btn-secondary btn-sm">{{ $t('common.reset')
          }}</button>

        <button @click="initListController()" :title="$t('device.reloadShort')" :disabled="loading"
          class="btn btn-secondary !px-2.5 !min-h-[40px]" :aria-label="$t('device.reloadShort')">
          <font-awesome :icon="faRotate" class="w-3.5 h-3.5" :class="loading ? 'animate-spin' : ''" />
        </button>
      </div>
    </div>

    <!-- loading -->
    <div v-if="loading" class="p-3 space-y-2">
      <div v-for="i in 3" :key="i" class="skeleton h-14 rounded-xl"></div>
    </div>

    <!-- desktop table -->
    <div v-else class="hidden md:block table-shell max-h-[460px]">
      <table class="w-full table-fixed">
        <thead class="sticky top-0 z-10">
          <tr>
            <th class="th w-[42%]">{{ $t('device.colDevice') }}</th>
            <th class="th w-[28%]">{{ $t('device.colSerialHandler') }}</th>
            <th class="th !text-center w-[18%] hidden xl:table-cell">{{ $t('device.colLastSync') }}</th>
            <th class="th !text-center w-[30%]">{{ $t('common.actions') }}</th>
          </tr>
        </thead>

        <tbody v-if="filtered.length > 0">
          <tr v-for="device in filtered" :key="device.id" class="even:bg-surface-raised/25">
            <td class="td">
              <div class="flex items-center gap-2.5 min-w-0">
                <span class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deep grid place-content-center shrink-0">
                  <font-awesome :icon="faServer" class="w-3.5 h-3.5" />
                </span>
                <div class="min-w-0">
                  <p class="text-[13px] font-semibold text-text truncate">{{ device.name || '-' }}</p>
                  <p class="mono text-[10px] text-text-muted truncate">
                    #{{ device.id }} · {{ tzLabel(device.timezone) }}<template v-if="device.ip"> · {{ device.ip
                      }}</template>
                  </p>
                  <p class="mt-0.5 inline-flex items-center gap-1.5 mono text-[10px] font-bold"
                    :class="statusMeta(device.status).tone">
                    <span class="w-1.5 h-1.5 rounded-full" :class="statusMeta(device.status).dot"></span>
                    {{ statusMeta(device.status).label }}
                    <span v-if="device.location" class="text-text-muted/80 font-normal truncate">· {{ device.location
                      }}</span>
                  </p>
                </div>
              </div>
            </td>

            <td class="td whitespace-nowrap">
              <p class="mono text-[11.5px] font-bold text-text">{{ device.sn }}</p>
              <p class="mono text-[10px] text-text-muted">{{ handlerLabel(device.handler) }}</p>
            </td>


            <td class="td !text-center hidden xl:table-cell mono text-[10.5px] text-text-muted whitespace-nowrap">
              {{ lastSyncText(device.last_sync) }}
            </td>

            <td class="td">
              <div class="flex items-center justify-center gap-0.5">
                <button @click="pullDevice(device.id)" :disabled="pulling[device.id]"
                  class="w-7 h-7 rounded-lg border border-border text-warn hover:bg-warn-bg transition grid place-content-center disabled:opacity-50"
                  :title="$t('device.pullData')" :aria-label="$t('device.pullData')">
                  <font-awesome :icon="faDownload" class="w-3 h-3" :class="pulling[device.id] ? 'animate-pulse' : ''" />
                </button>
                <button v-if="Number(device.status) === 3" @click="toggleDevice(device.id)"
                  class="w-7 h-7 rounded-lg border border-border text-primary-deep hover:bg-surface-raised transition grid place-content-center"
                  :title="$t('device.enableDevice')" :aria-label="$t('device.enableDevice')">
                  <font-awesome :icon="faRotate" class="w-3 h-3" />
                </button>
                <button v-else-if="canEdit" @click="toggleDevice(device.id)"
                  class="w-7 h-7 rounded-lg border border-border text-text-muted hover:bg-surface-raised transition grid place-content-center"
                  :title="$t('device.disableDevice')" :aria-label="$t('device.disableDevice')">
                  <font-awesome :icon="faBan" class="w-3 h-3" />
                </button>
                <button v-if="canEdit" @click="callbackEdit(device.id)"
                  class="w-7 h-7 rounded-lg border border-border text-primary-deep hover:bg-surface-raised transition grid place-content-center"
                  :title="$t('common.edit')" :aria-label="$t('common.edit')">
                  <font-awesome :icon="faPen" class="w-3 h-3" />
                </button>
                <button v-if="canDelete" @click="callbackDelete(device.id)"
                  class="w-7 h-7 rounded-lg border border-border text-danger hover:bg-danger-bg transition grid place-content-center"
                  :title="$t('common.delete')" :aria-label="$t('common.delete')">
                  <font-awesome :icon="faTrash" class="w-3 h-3" />
                </button>
              </div>
            </td>
          </tr>
        </tbody>

        <tbody v-else>
          <tr>
            <td colspan="4" class="p-0">
              <div class="state">
                <div class="state-icon">
                  <span class="hud-corner hud-corner-tl"></span>
                  <span class="hud-corner hud-corner-br"></span>
                  <font-awesome :icon="faServer" class="w-5 h-5" />
                </div>
                <p class="state-title">{{ hasFilter ? $t('device.noResults') : $t('device.noneYet') }}</p>
                <p class="state-hint">{{ hasFilter ? $t('device.tryKeyword') : $t('device.emptyHint') }}</p>
                <button v-if="hasFilter" @click="clearFilter()" class="btn btn-secondary btn-sm mt-1">{{
                  $t('common.resetFilters') }}</button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- mobile cards -->
    <div v-if="!loading" class="md:hidden p-2.5 space-y-2.5 max-h-[460px] overflow-auto">
      <div v-if="filtered.length === 0" class="state">
        <div class="state-icon"><font-awesome :icon="faServer" class="w-5 h-5" /></div>
        <p class="state-title">{{ $t('device.noDevices') }}</p>
        <p class="state-hint">{{ $t('device.emptyHint') }}</p>
      </div>

      <div v-for="device in filtered" :key="device.id" class="rounded-xl border border-border bg-surface p-3">
        <div class="flex items-start justify-between gap-2">
          <div class="flex items-center gap-2.5 min-w-0">
            <span class="w-8 h-8 rounded-lg bg-surface-raised text-primary-deep grid place-content-center shrink-0">
              <font-awesome :icon="faServer" class="w-3.5 h-3.5" />
            </span>
            <div class="min-w-0">
              <p class="text-[13px] font-semibold text-text truncate">{{ device.name || '-' }}</p>
              <p class="mono text-[10.5px] text-text-muted truncate">SN {{ device.sn }}</p>
            </div>
          </div>
          <span class="shrink-0 inline-flex items-center gap-1.5 mono text-[10px] font-bold"
            :class="statusMeta(device.status).tone">
            <span class="w-1.5 h-1.5 rounded-full" :class="statusMeta(device.status).dot"></span>
            {{ statusMeta(device.status).label }}
          </span>
        </div>

        <p class="mt-2 text-[11.5px] text-text-muted truncate">
          {{ device.location || $t('device.noLocation') }} <span class="mono">· {{ tzLabel(device.timezone) }}</span>
        </p>
        <p class="mono text-[10px] text-text-muted/80 mt-0.5">{{ $t('device.stSync') }} {{
          lastSyncText(device.last_sync) }}
          · #{{ device.id }}</p>

        <div class="mt-2.5 flex flex-wrap gap-1.5">
          <button @click="pullDevice(device.id)" :disabled="pulling[device.id]"
            class="btn btn-secondary btn-sm flex-1">{{
              pulling[device.id] ? $t('device.pulling') : $t('device.pullData') }}</button>
          <button v-if="Number(device.status) === 3" @click="toggleDevice(device.id)"
            class="btn btn-primary btn-sm flex-1">{{ $t('device.enableDevice') }}</button>
          <button v-else-if="canEdit" @click="toggleDevice(device.id)" class="btn btn-secondary btn-sm flex-1">{{
            $t('device.disableDevice') }}</button>
          <button v-if="canEdit" @click="callbackEdit(device.id)" class="btn btn-primary btn-sm flex-1">{{
            $t('common.edit')
            }}</button>
          <button v-if="canDelete" @click="callbackDelete(device.id)" class="btn btn-danger btn-sm flex-1">{{
            $t('common.delete') }}</button>
        </div>
      </div>
    </div>

    <div class="px-4 py-2 border-t border-border flex flex-wrap items-center gap-2 mono text-[10px] text-text-muted">
      <span>{{ $t('device.countRegistered', { n: contents.length }) }}</span>
      <span class="opacity-40">|</span>
      <span>{{ $t('device.countFiltered', { filtered: filtered.length, total: contents.length }) }}</span>
      <span v-if="hasFilter" class="ml-auto chip chip-warn">{{ $t('common.activeFilters') }}</span>
    </div>
  </div>
</template>
