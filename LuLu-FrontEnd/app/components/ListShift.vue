<script setup>
import { useAuthStore } from '~/store/auth';
import { useChecker } from '#imports';
import { useShiftStore } from '~/store/shift';
import { faPen, faTrash, faGrip, faInbox } from '@fortawesome/free-solid-svg-icons';

const
  auth = useAuthStore(),
  config = useRuntimeConfig(),
  list = useShiftStore(),
  check = useChecker(),
  emit = defineEmits(['edit', 'delete']),
  contents = computed(() => list.contents);

// Declared after `auth` exists: a computed referencing `auth` above its
// declaration throws on first access, not at definition time.
const canEdit = computed(() => auth.hasActionAccess('shift', 'can_edit'))
const canDelete = computed(() => auth.hasActionAccess('shift', 'can_delete'))

function callbackEdit(row) { emit('edit', row) }
function callbackDelete(row) { emit('delete', row) }

async function getListShift() {
  try {
    const response = await apiFetch(`${config.public.apiBase}/shift/list`, { method: "GET", headers: auth.confHeaders() });
    if (response.code === 200) list.set(response.data);
  } catch (e) { console.error(e) }
}

function fmt(sec) {
  const s = ((sec % 86400) + 86400) % 86400
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`
}
function shiftColor(s) {
  const h = Math.floor(((s.start_time % 86400) + 86400) % 86400 / 3600)
  if (h >= 5 && h < 12) return 'bg-amber-400'
  if (h >= 12 && h < 18) return 'bg-emerald-500'
  if (h >= 18 || h < 5) return 'bg-emerald-500'
  return 'bg-emerald-500'
}
function handleDragStart(e, row) {
  e.dataTransfer.setData('shift', JSON.stringify({ act: 'copy', id: row.id }))
  e.dataTransfer.effectAllowed = 'copy'
  // ghost
  if (e.dataTransfer.setDragImage) {
    const el = document.createElement('div')
    el.textContent = row.name
    el.style.position = 'absolute'; el.style.top = '-1000px'
    el.className = 'px-3 py-1 rounded-full bg-emerald-600 text-white text-xs'
    document.body.appendChild(el)
    e.dataTransfer.setDragImage(el, 0, 0)
    setTimeout(() => el.remove(), 0)
  }
}

onMounted(() => getListShift());
defineExpose({ reload: getListShift })
</script>

<template>
  <div class="flex flex-col">

    <!-- ===== desktop table ===== -->
    <div class="hidden md:block table-shell">
      <table class="w-full min-w-[360px] border-collapse hidden md:table">
        <thead class="sticky top-0 z-10">
          <tr>
            <th class="th w-8"></th>
            <th class="th">{{ $t('shift.colShift') }}</th>
            <th class="th !text-center">{{ $t('shift.colWorkHours') }}</th>
            <th class="th !text-center hidden 2xl:table-cell">{{ $t('shift.colPunchWindow') }}</th>
            <th class="th !text-center">{{ $t('common.actions') }}</th>
          </tr>
        </thead>

        <tbody v-if="contents.length > 0">
          <tr v-for="row in contents" :key="row.id" draggable="true" @dragstart="(e) => handleDragStart(e, row)"
            class="even:bg-surface-raised/25">
            <td class="td">
              <span
                class="inline-flex w-7 h-7 rounded-lg border border-border bg-surface text-text-muted items-center justify-center cursor-grab active:cursor-grabbing">
                <font-awesome :icon="faGrip" class="w-3 h-3" />
              </span>
            </td>

            <td class="td">
              <div class="flex items-center gap-2.5 min-w-0">
                <span class="w-1.5 h-8 rounded-full bg-primary shrink-0"></span>
                <div class="min-w-0">
                  <p class="text-[13px] font-semibold text-text truncate">{{ row.name }}</p>
                  <div class="mt-0.5 flex flex-wrap items-center gap-1">
                    <span class="mono text-[10px] text-text-muted">#{{ row.id }}</span>
                    <span v-if="row.nextday" class="chip chip-quiet">{{ $t('shift.nextDay') }}</span>
                    <span v-if="row.prevday" class="chip chip-warn">{{ $t('shift.prevDay') }}</span>
                  </div>
                </div>
              </div>
            </td>

            <td class="td whitespace-nowrap">
              <p class="num text-[12.5px] font-bold text-text">{{ fmt(row.start_time) }} - {{ fmt(row.end_time) }}</p>
            </td>

            <td class="td hidden 2xl:table-cell whitespace-nowrap">
              <p class="mono text-[10.5px] text-text-muted">{{ fmt(row.start_enroll) }} - {{ fmt(row.start_time) }}</p>
              <p class="mono text-[10.5px] text-text-muted">{{ fmt(row.end_time) }} - {{ fmt(row.end_enroll) }}</p>
            </td>

            <td class="td">
              <div class="flex items-center justify-center gap-1.5">
                <button v-if="canEdit" @click="callbackEdit(row)"
                  class="w-9 h-9 rounded-lg border border-border text-text-muted hover:bg-surface-raised hover:text-primary-deeper transition grid place-content-center"
                  :title="$t('common.edit')" :aria-label="$t('common.edit')">
                  <font-awesome :icon="faPen" class="w-3.5 h-3.5" />
                </button>
                <button v-if="canDelete" @click="callbackDelete(row)"
                  class="w-9 h-9 rounded-lg border border-border text-danger hover:bg-danger-bg transition grid place-content-center"
                  :title="$t('common.delete')" :aria-label="$t('common.delete')">
                  <font-awesome :icon="faTrash" class="w-3.5 h-3.5" />
                </button>
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
                <p class="state-title">{{ $t('shift.emptyShort') }}</p>
                <p class="state-hint">{{ $t('shift.formHint') }}</p>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- ===== mobile cards ===== -->
    <div class="md:hidden p-2.5 space-y-2.5 max-h-[420px] overflow-auto">
      <div v-if="contents.length === 0" class="state">
        <div class="state-icon">
          <font-awesome :icon="faInbox" class="w-5 h-5" />
        </div>
        <p class="state-title">{{ $t('shift.emptyShort') }}</p>
        <p class="state-hint">{{ $t('shift.formHint') }}</p>
      </div>

      <div v-for="row in contents" :key="row.id" draggable="true" @dragstart="(e) => handleDragStart(e, row)"
        class="rounded-xl border border-border bg-surface p-3 active:border-primary/40 active:bg-surface-raised/40">
        <div class="flex items-start gap-2.5">
          <span class="w-1.5 h-10 rounded-full bg-primary shrink-0"></span>
          <div class="flex-1 min-w-0">
            <p class="text-[13px] font-semibold text-text truncate">{{ row.name }}</p>
            <p class="mono text-[10px] text-text-muted mt-0.5">#{{ row.id }}</p>
          </div>
          <font-awesome :icon="faGrip" class="w-3.5 h-3.5 text-text-muted shrink-0 mt-1" />
        </div>

        <div class="mt-2.5 grid grid-cols-2 gap-2">
          <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
            <p class="mono text-[9.5px] tracking-[0.1em] uppercase text-text-muted">{{ $t('shift.colWorkHours') }}</p>
            <p class="num text-[11.5px] font-bold text-text mt-0.5">{{ fmt(row.start_time) }} - {{ fmt(row.end_time) }}
            </p>
          </div>
          <div class="rounded-lg bg-canvas border border-border px-2.5 py-2">
            <p class="mono text-[9.5px] tracking-[0.1em] uppercase text-text-muted">{{ $t('shift.colPunchWindow') }}</p>
            <p class="mono text-[10.5px] text-text mt-0.5">{{ fmt(row.start_enroll) }} - {{ fmt(row.start_time) }}</p>
            <p class="mono text-[10.5px] text-text">{{ fmt(row.end_time) }} - {{ fmt(row.end_enroll) }}</p>
          </div>
        </div>

        <div v-if="row.nextday || row.prevday" class="mt-2 flex flex-wrap gap-1">
          <span v-if="row.nextday" class="chip chip-quiet">{{ $t('shift.nextDay') }}</span>
          <span v-if="row.prevday" class="chip chip-warn">{{ $t('shift.prevDay') }}</span>
        </div>

        <div class="mt-2.5 flex flex-wrap gap-1.5">
          <button v-if="canEdit" @click="callbackEdit(row)" class="btn btn-primary flex-1">{{ $t('common.edit')
            }}</button>
          <button v-if="canDelete" @click="callbackDelete(row)" class="btn btn-danger flex-1">{{ $t('common.delete')
            }}</button>
        </div>
      </div>
    </div>

  </div>
</template>
